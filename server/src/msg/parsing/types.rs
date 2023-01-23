
pub trait WSMessage
{
    fn serialize(&self) -> Vec<u8>;
    fn deserialize(data: &Vec<u8>) -> anyhow::Result<Self> where Self: Sized;
}