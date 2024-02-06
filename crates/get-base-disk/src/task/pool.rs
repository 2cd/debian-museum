use log::info;
use std::{self, sync::OnceLock};
use threadpool::ThreadPool;

pub(crate) static POOL: OnceLock<ThreadPool> = OnceLock::new();

pub(crate) fn global_pool() -> &'static ThreadPool {
    POOL.get_or_init(ThreadPool::default)
}

pub(crate) fn wait_process(children: Vec<(&str, std::process::Child)>) {
    for (name, mut task) in children {
        if task
            .try_wait()
            .is_ok_and(|x| x.is_some())
        {
            continue;
        }

        info!("wait process: {}, id: {}", name, task.id());

        if let Err(e) = task.wait() {
            log::error!("Task: {name}, Err: {e}");
        }
    }
}
