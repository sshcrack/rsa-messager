#[derive(Copy, Clone)]
pub enum Modes {
    SetPubkey,
    To,
    From,
    Name,
    WantUid,
    UidReply,
}

impl Modes {
    pub fn get_indicator(self) -> u8 {
        match self {
            Self::SetPubkey => 0,
            Self::To => 1,
            Self::From => 2,
            Self::Name => 3,
            Self::WantUid => 4,
            Self::UidReply => 5,
        }
    }

    pub fn is_indicator(self, b: &u8) -> bool {
        let ind = self.get_indicator();
        return ind.eq(b);
    }

    pub fn get_send(self, end: &Vec<u8>) -> Vec<u8> {
        let ind = self.get_indicator();

        let mut el = end.to_vec().clone();
        el.reverse();

        el.push(ind);

        el.reverse();

        return el;
    }
}
