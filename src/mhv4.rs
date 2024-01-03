use serde::Serialize;

#[derive(Serialize, Debug, Clone, Copy)]
pub struct MHV4Data {
    idc: usize,
    bus: usize,
    dev: usize,
    ch: usize,
    current: usize,
    target: usize,
    is_on: bool,
    is_rc: bool,
    is_changing: bool,
}

impl MHV4Data {
    pub fn new(
        in_idc: usize,
        in_bus: usize,
        in_dev: usize,
        in_ch: usize,
        in_is_on: bool,
        in_is_rc: bool,
    ) -> MHV4Data {
        MHV4Data {
            idc: in_idc,
            bus: in_bus,
            dev: in_dev,
            ch: in_ch,
            current: 0,
            target: 0,
            is_on: in_is_on,
            is_rc: in_is_rc,
            is_changing: false,
        }
    }

    //pub fn get_idc(self) -> usize {
    //    self.idc
    //}

    pub fn get_module_id(self) -> (usize, usize, usize) {
        (self.bus, self.dev, self.ch)
    }
    // ここに関連する関数を定義
    // 例:
    // pub fn some_processing(&self) {
    //     // 処理内容
    // }
}
