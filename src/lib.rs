pub mod fat;
pub mod mini_fat;
pub mod difat;
pub mod directory;
pub mod user_defined_data;
pub mod range_lock;
pub mod common;
pub mod header;
pub mod ole;

#[cfg(test)]
mod tests {
    use binrw::BinRead;

    #[test]
    fn it_works() {
        let mut h = crate::ole::Ole::from_path("./abcd.doc").unwrap();
        h.parse().unwrap();
        let entry = &h.entries.as_ref().unwrap()[1];
        let data = h.read(entry).unwrap();
        println!("{:?}", data)
    }
}
