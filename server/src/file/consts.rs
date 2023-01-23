use lazy_static::lazy_static;

use super::types::*;

lazy_static! {
    pub static ref FILE_REQUESTS: FileRequestList = FileRequestList::default();

    //? Yes I know there is no plural of information alright
    pub static ref FILE_INFOS: FileInfoList = FileInfoList::default();
}