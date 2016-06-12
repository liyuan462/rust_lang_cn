macro_rules! m_hashmap {
    ($($key:ident => $v:expr),*) => {
        {
            let mut map = HashMap::new();

            $(
                map.insert($key, $v);
            )*

            map
        }
    }
}
