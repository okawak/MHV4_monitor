use serde::Serialize;

#[derive(Serialize, Debug, Clone, Copy)]
pub struct MHV4Data {
    pub idc: usize,
    pub bus: usize,
    pub dev: usize,
    pub ch: usize,
    current: isize,
    pub is_on: bool,
    pub is_positive: bool,
}

impl MHV4Data {
    pub fn new(
        in_idc: usize,
        in_bus: usize,
        in_dev: usize,
        in_ch: usize,
        in_current: isize,
        in_is_on: bool,
        in_is_positive: bool,
    ) -> MHV4Data {
        MHV4Data {
            idc: in_idc,
            bus: in_bus,
            dev: in_dev,
            ch: in_ch,
            current: in_current,
            is_on: in_is_on,
            is_positive: in_is_positive,
        }
    }

    pub fn get_module_id(self) -> (usize, usize, usize) {
        (self.bus, self.dev, self.ch)
    }

    pub fn get_current(self) -> isize {
        self.current
    }

    pub fn set_current(&mut self, in_current: isize) {
        self.current = in_current;
    }
}
