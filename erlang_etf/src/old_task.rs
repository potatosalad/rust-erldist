#[derive(Debug)]
pub enum TaskHint {
    Continue,
    Yield,
}

pub enum Task {
    Blocking,
    Yielding(YieldingTask),
}

impl Task {
    pub fn blocking() -> Self {
        Task::Blocking
    }

    pub fn yielding(reductions_per_timeslice: usize) -> Self {
        if reductions_per_timeslice > 0 {
            Task::Yielding(YieldingTask::new(reductions_per_timeslice))
        } else {
            Self::blocking()
        }
    }

    pub fn bump_all_reds(&mut self) -> TaskHint {
        match self {
            Task::Blocking => TaskHint::Continue,
            Task::Yielding(ref mut x) => x.bump_all_reds(),
        }
    }

    pub fn bump_reds(&mut self, reductions: usize) -> TaskHint {
        match self {
            Task::Blocking => TaskHint::Continue,
            Task::Yielding(ref mut x) => x.bump_reds(reductions),
        }
    }

    pub fn get_consumed_reds(&self) -> usize {
        match self {
            Task::Blocking => 0,
            Task::Yielding(x) => x.get_consumed_reds(),
        }
    }

    pub fn get_remaining_reds(&self) -> usize {
        match self {
            Task::Blocking => usize::MAX,
            Task::Yielding(x) => x.get_remaining_reds(),
        }
    }

    pub fn reset_after_yield(&mut self) {
        match self {
            Task::Blocking => (),
            Task::Yielding(ref mut x) => x.reset_after_yield(),
        }
    }

    pub fn should_yield(&self) -> bool {
        match self {
            Task::Blocking => false,
            Task::Yielding(x) => x.should_yield(),
        }
    }
}

pub struct YieldingTask {
    reductions_per_timeslice: usize,
    reds: usize,
}

impl YieldingTask {
    fn new(reductions_per_timeslice: usize) -> Self {
        YieldingTask {
            reductions_per_timeslice,
            reds: 0,
        }
    }

    fn bump_all_reds(&mut self) -> TaskHint {
        self.reds = 0;
        TaskHint::Yield
    }

    fn bump_reds(&mut self, reductions: usize) -> TaskHint {
        if self.reds <= reductions {
            self.reds = 0;
            TaskHint::Yield
        } else {
            self.reds -= reductions;
            TaskHint::Continue
        }
    }

    fn get_consumed_reds(&self) -> usize {
        self.reductions_per_timeslice - self.reds
    }

    fn get_remaining_reds(&self) -> usize {
        self.reds
    }

    fn reset_after_yield(&mut self) {
        self.reds = self.reductions_per_timeslice;
    }

    fn should_yield(&self) -> bool {
        self.reds == 0
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum TaskState<Y, R> {
    /// The generator suspended with a value.
    ///
    /// This state indicates that a generator has been suspended, and typically
    /// corresponds to a `yield` statement. The value provided in this variant
    /// corresponds to the expression passed to `yield` and allows generators to
    /// provide a value each time they yield.
    Yielded(Y),

    /// The generator completed with a return value.
    ///
    /// This state indicates that a generator has finished execution with the
    /// provided value. Once a generator has returned `Complete` it is
    /// considered a programmer error to call `resume` again.
    Complete(R),
}

// pub enum ReductionGeneratorState<Y, R> {
//     /// The generator suspended with a value.
//     ///
//     /// This state indicates that a generator has been suspended, and typically
//     /// corresponds to a `yield` statement. The value provided in this variant
//     /// corresponds to the expression passed to `yield` and allows generators to
//     /// provide a value each time they yield.
//     Yielded(Y),

//     /// The generator completed with a return value.
//     ///
//     /// This state indicates that a generator has finished execution with the
//     /// provided value. Once a generator has returned `Complete` it is
//     /// considered a programmer error to call `resume` again.
//     Complete(R),
// }

// pub trait ReductionGenerator {
//     type Yield;
//     type Return;

//     fn resume(self: Pin<&mut Self>, reductions: &mut Option<usize>) -> ReductionGeneratorState<Self::Yield, Self::Return>;
// }
