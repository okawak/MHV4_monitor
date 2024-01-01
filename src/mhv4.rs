use serde::Serialize;

#[derive(Serialize)]
pub struct MHV4Data {
    idc: usize,
    bus: usize,
    dev: usize,
    ch: usize,
    current: usize,
    target: usize,
}

impl MHV4Data {
    pub fn new(in_idc: usize, in_bus: usize, in_dev: usize, in_ch: usize) -> MHV4Data {
        MHV4Data {
            idc: in_idc,
            bus: in_bus,
            dev: in_dev,
            ch: in_ch,
            current: 0,
            target: 0,
        }
    }

    // ここに関連する関数を定義
    // 例:
    // pub fn some_processing(&self) {
    //     // 処理内容
    // }
}
