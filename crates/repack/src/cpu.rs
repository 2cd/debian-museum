use std::{num::NonZeroUsize, sync::OnceLock, thread};

pub fn num() -> &'static usize {
    static CPU_NUM: OnceLock<usize> = OnceLock::new();
    CPU_NUM.get_or_init(|| {
        usize::from(
            thread::available_parallelism()
                .unwrap_or_else(|_| unsafe { NonZeroUsize::new_unchecked(1) }),
        )
    })
}
