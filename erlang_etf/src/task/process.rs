use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::Arc;
// use std::cell::UnsafeCell;
// use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum Process {
    Blocking,
    Yielding(ReductionCountingProcess),
}
impl Process {
    pub fn blocking() -> Self {
        Self::Blocking
    }

    pub fn yielding() -> Self {
        Self::Yielding(ReductionCountingProcess::new())
    }

    pub async fn bump_all_reds(&self) {
        match self {
            Process::Blocking => (),
            Process::Yielding(x) => x.bump_all_reds().await,
        };
    }

    pub async fn bump_reds(&self, gc: isize) {
        match self {
            Process::Blocking => (),
            Process::Yielding(x) => x.bump_reds(gc).await,
        };
    }

    pub fn get_consumed_reds(&self) -> isize {
        match self {
            Process::Blocking => 0,
            Process::Yielding(x) => x.get_consumed_reds(),
        }
    }

    pub fn get_remaining_reds(&self) -> isize {
        match self {
            Process::Blocking => 0,
            Process::Yielding(x) => x.get_remaining_reds(),
        }
    }

    pub fn get_timeslice_pct(&self) -> isize {
        match self {
            Process::Blocking => 1,
            Process::Yielding(x) => x.get_timeslice_pct(),
        }
    }

    // pub fn is_yielded(&self) -> bool {
    //     match self {
    //         Process::Blocking => false,
    //         Process::Yielding(x) => x.is_yielded(),
    //     }
    // }

    // pub async fn yield_now(&self) {
    //     match self {
    //         Process::Blocking => (),
    //         Process::Yielding(x) => x.yield_now().await,
    //     };
    // }
}

// #[derive(Clone, Debug)]
// pub struct YieldingProcess(Rc<UnsafeCell<ReductionCountingProcess>>);
// impl YieldingProcess {
//     pub fn new() -> Self {
//         Self(Rc::new(UnsafeCell::new(ReductionCountingProcess::new())))
//     }

//     pub async fn bump_all_reds(&self) {
//         unsafe { (&mut *self.0.get()).bump_all_reds(); };
//         self.maybe_yield().await;
//     }

//     pub async fn bump_reds(&self, gc: isize) {
//         unsafe { (&mut *self.0.get()).bump_reds(gc); };
//         self.maybe_yield().await;
//     }

//     pub fn get_consumed_reds(&self) -> isize {
//         unsafe { (&*self.0.get()).get_consumed_reds() }
//     }

//     pub fn get_remaining_reds(&self) -> isize {
//         unsafe { (&*self.0.get()).get_remaining_reds() }
//     }

//     pub fn get_timeslice_pct(&self) -> isize {
//         unsafe { (&*self.0.get()).get_timeslice_pct() }
//     }

//     pub fn is_yielded(&self) -> bool {
//         unsafe { (&*self.0.get()).is_yielded() }
//     }

//     pub async fn yield_now(&self) {
//         cassette::yield_now().await;
//         self.reset_reds();
//     }

//     async fn maybe_yield(&self) {
//         if self.is_yielded() {
//             self.yield_now().await;
//         }
//     }

//     fn reset_reds(&self) {
//         unsafe { (&mut *self.0.get()).reset_reds(); };
//     }
// }

#[derive(Clone, Debug)]
pub struct ReductionCountingProcess {
    fcalls: Arc<AtomicIsize>,
}
impl ReductionCountingProcess {
    const CONTEXT_REDS: isize = 4000;

    pub fn new() -> Self {
        Self {
            fcalls: Arc::new(AtomicIsize::new(0)),
        }
    }

    pub async fn bump_all_reds(&self) {
        self.fcalls.store(-Self::CONTEXT_REDS, Ordering::Relaxed);
        cassette::yield_now().await;
        self.fcalls.store(0, Ordering::Relaxed);
    }

    pub async fn bump_reds(&self, gc: isize) {
        if self.fcalls.fetch_sub(gc, Ordering::Relaxed) - gc <= -Self::CONTEXT_REDS {
            self.bump_all_reds().await;
        }
    }

    pub fn get_consumed_reds(&self) -> isize {
        -self.fcalls.load(Ordering::Relaxed)
    }

    pub fn get_remaining_reds(&self) -> isize {
        Self::CONTEXT_REDS + self.fcalls.load(Ordering::Relaxed)
    }

    pub fn get_timeslice_pct(&self) -> isize {
        let mut pct = (self.get_consumed_reds() * 100) / (Self::CONTEXT_REDS);
        if pct < 1 {
            pct = 1;
        } else if pct > 100 {
            pct = 100;
        }
        pct
    }
}
