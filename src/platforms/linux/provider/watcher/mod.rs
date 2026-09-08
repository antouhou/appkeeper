use super::super::paths::SearchPaths;
use super::catalog::{self, AppCatalog};
use notify::event::{AccessKind, AccessMode};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::thread::{self, JoinHandle};

fn request_refresh(sender: &SyncSender<()>) {
    let _ = sender.try_send(());
}

fn relevant(event: &Event, paths: &SearchPaths) -> bool {
    if event.need_rescan() {
        return true;
    }
    if matches!(event.kind, EventKind::Access(_))
        && !matches!(
            event.kind,
            EventKind::Access(AccessKind::Close(AccessMode::Write))
        )
    {
        return false;
    }
    event.paths.iter().any(|path| {
        paths
            .application_dirs()
            .iter()
            .any(|root| path.starts_with(root) || root.starts_with(path))
    })
}

fn update_watches(
    native: &mut RecommendedWatcher,
    paths: &SearchPaths,
    watches: &mut BTreeMap<PathBuf, RecursiveMode>,
) {
    let mut desired = BTreeMap::new();
    for root in paths.application_dirs() {
        // Watch each parent before checking its child. A child created after
        // that check must leave a notification for the next scan.
        for ancestor in root.ancestors().collect::<Vec<_>>().into_iter().rev() {
            if !ancestor.is_dir() {
                break;
            }
            let path = ancestor.to_path_buf();
            if desired.contains_key(&path) {
                continue;
            }
            let mode = if paths.application_dirs().contains(&path) {
                RecursiveMode::Recursive
            } else {
                RecursiveMode::NonRecursive
            };
            match native.watch(&path, mode) {
                Ok(()) => {
                    desired.insert(path, mode);
                }
                Err(error) => {
                    tracing::warn!(path = %path.display(), %error, "cannot watch application search path")
                }
            }
        }
    }
    for path in watches.keys().filter(|path| !desired.contains_key(*path)) {
        // The kernel may already have removed a watch for a deleted directory.
        let _ = native.unwatch(path);
    }
    *watches = desired;
}

fn watch_catalog(
    paths: &SearchPaths,
    stopping: &AtomicBool,
    requests: &Receiver<()>,
    native: &mut RecommendedWatcher,
    watches: &mut BTreeMap<PathBuf, RecursiveMode>,
    publish: &mut impl FnMut(AppCatalog, &AtomicBool),
) {
    loop {
        if stopping.load(Ordering::Acquire) {
            return;
        }
        update_watches(native, paths, watches);
        match catalog::load_apps(paths) {
            Ok(apps) if !stopping.load(Ordering::Acquire) => publish(apps, stopping),
            Ok(_) => return,
            Err(error) => {
                tracing::warn!(%error, "keeping previous application catalog after scan failure")
            }
        }
        if requests.recv().is_err() {
            return;
        }
    }
}

pub(super) struct CatalogWatcher {
    refresh: SyncSender<()>,
    stopping: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}

impl CatalogWatcher {
    pub(super) fn start(
        paths: Arc<SearchPaths>,
        mut publish: impl FnMut(AppCatalog, &AtomicBool) + Send + 'static,
    ) -> notify::Result<Self> {
        let (refresh, requests) = mpsc::sync_channel(1);
        let callback_refresh = refresh.clone();
        let callback_paths = Arc::clone(&paths);
        let mut native =
            notify::recommended_watcher(move |result: notify::Result<Event>| match result {
                Ok(event) if relevant(&event, &callback_paths) => {
                    request_refresh(&callback_refresh)
                }
                Ok(_) => {}
                Err(error) => {
                    tracing::warn!(%error, "application watcher reported an error");
                    request_refresh(&callback_refresh);
                }
            })?;
        let mut watches = BTreeMap::new();
        update_watches(&mut native, &paths, &mut watches);
        let stopping = Arc::new(AtomicBool::new(false));
        let worker_stopping = Arc::clone(&stopping);
        let worker = thread::Builder::new()
            .name("appkeeper-catalog".into())
            .spawn(move || {
                watch_catalog(
                    &paths,
                    &worker_stopping,
                    &requests,
                    &mut native,
                    &mut watches,
                    &mut publish,
                );
            })
            .map_err(notify::Error::io)?;
        Ok(Self {
            refresh,
            stopping,
            worker: Some(worker),
        })
    }
}

impl Drop for CatalogWatcher {
    fn drop(&mut self) {
        self.stopping.store(true, Ordering::Release);
        request_refresh(&self.refresh);
        if let Some(worker) = self.worker.take()
            && worker.thread().id() != thread::current().id()
            && worker.join().is_err()
        {
            tracing::error!("application catalog worker panicked");
        }
    }
}

#[cfg(test)]
mod tests;
