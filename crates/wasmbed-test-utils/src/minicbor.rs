use minicbor::encode::Encode;
use minicbor::decode::Decode;

pub fn assert_encode_decode<T>(v: &T)
where
    T: PartialEq + std::fmt::Debug + Encode<()> + for<'b> Decode<'b, ()>,
{
    let encoded = minicbor::to_vec(v).unwrap();
    let decoded = minicbor::decode(&encoded).unwrap();
    assert_eq!(*v, decoded);
}
