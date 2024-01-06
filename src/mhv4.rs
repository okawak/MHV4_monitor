use serde::Serialize;

#[derive(Serialize, Debug, Clone, Copy)]
pub struct MHV4Data {
    idc: usize,
    bus: usize,
    dev: usize,
    ch: usize,
    current: isize,
}

impl MHV4Data {
    pub fn new(
        in_idc: usize,
        in_bus: usize,
        in_dev: usize,
        in_ch: usize,
        in_current: isize,
    ) -> MHV4Data {
        MHV4Data {
            idc: in_idc,
            bus: in_bus,
            dev: in_dev,
            ch: in_ch,
            current: in_current,
        }
    }

    #[allow(dead_code)]
    pub fn get_idc(self) -> usize {
        self.idc
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
