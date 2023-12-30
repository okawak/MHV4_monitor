use serde::Serialize;

#[derive(Serialize)]
pub struct MHV4Data {
    IDC: isize,
    bus: isize,
    dev: isize,
    target: isize,
}

impl MHV4Data {
    pub fn new() -> MHV4Data {
        MHV4Data {
            IDC: -1,
            bus: -1,
            dev: -1,
            target: -1,
        }
    }

    // ここに関連する関数を定義
    // 例:
    // pub fn some_processing(&self) {
    //     // 処理内容
    // }
}
