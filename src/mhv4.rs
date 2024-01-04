use serde::Serialize;

#[derive(Serialize, Debug, Clone, Copy)]
pub struct MHV4Data {
    idc: usize,
    bus: usize,
    dev: usize,
    ch: usize,
    current: isize,
    is_on: bool,
    is_rc: bool,
}

impl MHV4Data {
    pub fn new(
        in_idc: usize,
        in_bus: usize,
        in_dev: usize,
        in_ch: usize,
        in_current: isize,
        in_is_on: bool,
        in_is_rc: bool,
    ) -> MHV4Data {
        MHV4Data {
            idc: in_idc,
            bus: in_bus,
            dev: in_dev,
            ch: in_ch,
            current: in_current,
            is_on: in_is_on,
            is_rc: in_is_rc,
        }
    }

    #[allow(dead_code)]
    pub fn get_idc(self) -> usize {
        self.idc
    }

    pub fn get_module_id(self) -> (usize, usize, usize) {
        (self.bus, self.dev, self.ch)
    }

    pub fn get_status(self) -> (bool, bool) {
        (self.is_on, self.is_rc)
    }

    pub fn get_current(self) -> isize {
        self.current
    }

    pub fn set_status(&mut self, in_is_on: bool, in_is_rc: bool) {
        self.is_on = in_is_on;
        self.is_rc = in_is_rc;
    }
}
