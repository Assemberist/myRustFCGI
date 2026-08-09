mod planner_state;

pub use planner_state::{
    STATUS_STOPPED,
    STATUS_STARTED,
    STATUS_SUCCEED,
    STATUS_FAILED,
};

use planner_state::PlannerState;

pub const TASK_NEXT_PARTIAL_REQUEST: u8             = 0;    // find next partial request which awaiting for data
pub const TASK_NEXT_ARRIVED_DATA: u8                = 1;    // not need to find, since this is RB. Just check if Client still alive
pub const TASK_CHECK_REQUEST_TIMEOUT: u8            = 2;
pub const TASK_CHECK_CLIENT_TIMEOUT: u8             = 3;
pub const TASK_NEXT_FREE_REQUEST_CELL: u8           = 4;
pub const TASK_NEXT_FREE_PARTIAL_REQUEST_CELL: u8   = 5;
pub const TASK_NEXT_CLIENT: u8                      = 6;

pub struct TaskPlanner {
    state: PlannerState,
}

impl TaskPlanner {
    // Don't use inside planner
    // can break thread safity
    pub fn start_task(&self, task: u8) { self.state.start_task(task); }
    pub fn stop_task(&self, task: u8) -> bool { self.state.stop_task(task) }

    fn _loop(&mut self){
        let mut coin = true;
        loop {
            if let Some(task) = self.state.get_next_task() {
                let result = 
                    if coin { STATUS_SUCCEED }
                    else { STATUS_FAILED };
                
                self.state.set_task_result(task as u8, result);

                coin = !coin;
            }
        }
    }
}