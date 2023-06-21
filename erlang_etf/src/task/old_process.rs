use std::cell::RefCell;
use std::sync::Arc;

#[derive(Clone)]
pub enum Process {
    Blocking,
    Yielding(YieldingProcess),
}
impl Process {
    pub fn blocking() -> Self {
        Self::Blocking
    }

    pub fn yielding() -> Self {
        Self::Yielding(YieldingProcess::new())
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

    pub fn is_yielded(&self) -> bool {
        match self {
            Process::Blocking => false,
            Process::Yielding(x) => x.is_yielded(),
        }
    }

    pub async fn yield_now(&self) {
        match self {
            Process::Blocking => (),
            Process::Yielding(x) => x.yield_now().await,
        };
    }
}

#[derive(Clone)]
pub struct YieldingProcess(Arc<RefCell<ReductionCountingInner>>);
impl YieldingProcess {
    pub fn new() -> Self {
        Self(Arc::new(RefCell::new(ReductionCountingInner::new())))
    }

    pub async fn bump_all_reds(&self) {
        self.0.as_ref().borrow_mut().bump_all_reds();
        self.maybe_yield().await;
    }

    pub async fn bump_reds(&self, gc: isize) {
        self.0.as_ref().borrow_mut().bump_reds(gc);
        self.maybe_yield().await;
    }

    pub fn get_consumed_reds(&self) -> isize {
        self.0.as_ref().borrow().get_consumed_reds()
    }

    pub fn get_remaining_reds(&self) -> isize {
        self.0.as_ref().borrow().get_remaining_reds()
    }

    pub fn get_timeslice_pct(&self) -> isize {
        self.0.as_ref().borrow().get_timeslice_pct()
    }

    pub fn is_yielded(&self) -> bool {
        self.0.as_ref().borrow().is_yielded()
    }

    pub async fn yield_now(&self) {
        cassette::yield_now().await;
        self.reset_reds();
    }

    async fn maybe_yield(&self) {
        if self.is_yielded() {
            self.yield_now().await;
        }
    }

    fn reset_reds(&self) {
        self.0.as_ref().borrow_mut().reset_reds()
    }
}

struct ReductionCountingInner {
    fcalls: isize,
}
impl ReductionCountingInner {
    const CONTEXT_REDS: isize = 4000;

    fn new() -> Self {
        Self { fcalls: 0 }
    }

    fn bump_all_reds(&mut self) {
        self.fcalls = -Self::CONTEXT_REDS;
    }

    fn bump_reds(&mut self, gc: isize) {
        self.fcalls -= gc;
        if self.fcalls < -Self::CONTEXT_REDS {
            self.fcalls = -Self::CONTEXT_REDS;
        }
    }

    fn get_consumed_reds(&self) -> isize {
        -self.fcalls
    }

    fn get_remaining_reds(&self) -> isize {
        Self::CONTEXT_REDS + self.fcalls
    }

    fn get_timeslice_pct(&self) -> isize {
        let mut pct = (self.fcalls * 100) / (-Self::CONTEXT_REDS);
        if pct < 1 {
            pct = 1;
        } else if pct > 100 {
            pct = 100;
        }
        pct
    }

    fn is_yielded(&self) -> bool {
        self.fcalls <= -Self::CONTEXT_REDS
    }

    fn reset_reds(&mut self) {
        self.fcalls = 0;
    }
}
