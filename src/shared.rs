use crate::mhv4::MHV4Data;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct SharedData {
    mhv4_data_array: Vec<MHV4Data>,
    is_on: bool,
    is_rc: bool,
    is_progress: bool,
}

impl SharedData {
    pub fn new(in_vec: Vec<MHV4Data>, in_is_on: bool, in_is_rc: bool) -> SharedData {
        SharedData {
            mhv4_data_array: in_vec,
            is_on: in_is_on,
            is_rc: in_is_rc,
            is_progress: false,
        }
    }

    pub fn get_data(&self) -> Vec<MHV4Data> {
        self.mhv4_data_array.clone()
    }

    pub fn get_status(&self) -> (bool, bool) {
        (self.is_on, self.is_rc)
    }

    pub fn get_progress(&self) -> bool {
        self.is_progress
    }

    pub fn set_status(&mut self, in_is_on: bool, in_is_rc: bool) {
        self.is_on = in_is_on;
        self.is_rc = in_is_rc;
    }

    pub fn set_progress(&mut self, in_is_progress: bool) {
        self.is_progress = in_is_progress;
    }

    pub fn set_current(&mut self, id: usize, in_current: isize) {
        self.mhv4_data_array[id].set_current(in_current);
    }
}
