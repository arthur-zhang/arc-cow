use std::{
    borrow::Cow,
    cmp::Ordering,
    fmt::{self, Debug},
    hash::{Hash, Hasher},
    sync::Arc,
};

pub enum ArcCow<'a, T: ?Sized> {
    Borrowed(&'a T),
    Owned(Arc<T>),
}

impl<T: ?Sized + PartialEq> PartialEq for ArcCow<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        let a = self.as_ref();
        let b = other.as_ref();
        a == b
    }
}

impl<T: ?Sized + PartialOrd> PartialOrd for ArcCow<'_, T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl<T: ?Sized + Ord> Ord for ArcCow<'_, T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl<T: ?Sized + Eq> Eq for ArcCow<'_, T> {}

impl<T: ?Sized + Hash> Hash for ArcCow<'_, T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Borrowed(borrowed) => Hash::hash(borrowed, state),
            Self::Owned(owned) => Hash::hash(&**owned, state),
        }
    }
}

impl<T: ?Sized> Clone for ArcCow<'_, T> {
    fn clone(&self) -> Self {
        match self {
            Self::Borrowed(borrowed) => Self::Borrowed(borrowed),
            Self::Owned(owned) => Self::Owned(owned.clone()),
        }
    }
}

impl<'a, T: ?Sized> From<&'a T> for ArcCow<'a, T> {
    fn from(s: &'a T) -> Self {
        Self::Borrowed(s)
    }
}

impl<T: ?Sized> From<Arc<T>> for ArcCow<'_, T> {
    fn from(s: Arc<T>) -> Self {
        Self::Owned(s)
    }
}

impl<T: ?Sized> From<&'_ Arc<T>> for ArcCow<'_, T> {
    fn from(s: &'_ Arc<T>) -> Self {
        Self::Owned(s.clone())
    }
}

impl From<String> for ArcCow<'_, str> {
    fn from(value: String) -> Self {
        Self::Owned(value.into())
    }
}

impl From<&String> for ArcCow<'_, str> {
    fn from(value: &String) -> Self {
        Self::Owned(value.clone().into())
    }
}

impl<'a> From<Cow<'a, str>> for ArcCow<'a, str> {
    fn from(value: Cow<'a, str>) -> Self {
        match value {
            Cow::Borrowed(borrowed) => Self::Borrowed(borrowed),
            Cow::Owned(owned) => Self::Owned(owned.into()),
        }
    }
}

impl<T> From<Vec<T>> for ArcCow<'_, [T]> {
    fn from(vec: Vec<T>) -> Self {
        ArcCow::Owned(Arc::from(vec))
    }
}

impl<'a> From<&'a str> for ArcCow<'a, [u8]> {
    fn from(s: &'a str) -> Self {
        ArcCow::Borrowed(s.as_bytes())
    }
}

impl<T: ?Sized + ToOwned> std::borrow::Borrow<T> for ArcCow<'_, T> {
    fn borrow(&self) -> &T {
        match self {
            ArcCow::Borrowed(borrowed) => borrowed,
            ArcCow::Owned(owned) => owned.as_ref(),
        }
    }
}

impl<T: ?Sized> std::ops::Deref for ArcCow<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            ArcCow::Borrowed(s) => s,
            ArcCow::Owned(s) => s.as_ref(),
        }
    }
}

impl<T: ?Sized> AsRef<T> for ArcCow<'_, T> {
    fn as_ref(&self) -> &T {
        match self {
            ArcCow::Borrowed(borrowed) => borrowed,
            ArcCow::Owned(owned) => owned.as_ref(),
        }
    }
}

impl<T: ?Sized + Debug> Debug for ArcCow<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ArcCow::Borrowed(borrowed) => Debug::fmt(borrowed, f),
            ArcCow::Owned(owned) => Debug::fmt(&**owned, f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{borrow::Cow, collections::hash_map::DefaultHasher, collections::HashMap, hash::Hasher};

    #[test]
    fn test_creation_from_reference() {
        let value = 42;
        let arc_cow = ArcCow::from(&value);
        assert!(matches!(arc_cow, ArcCow::Borrowed(_)));
        assert_eq!(*arc_cow, 42);
    }

    #[test]
    fn test_creation_from_arc() {
        let value = Arc::new("Hello".to_string());
        let arc_cow = ArcCow::from(value.clone());
        assert!(matches!(arc_cow, ArcCow::Owned(_)));
        assert_eq!(arc_cow.as_ref(), "Hello");
    }

    #[test]
    fn test_clone_borrowed() {
        let value = "borrowed";
        let arc_cow1 :ArcCow<'static, str>= ArcCow::from(value);
        let arc_cow2 = arc_cow1.clone();
        
        // 克隆后仍为 Borrowed
        assert!(matches!(arc_cow1, ArcCow::Borrowed(_)));
        assert!(matches!(arc_cow2, ArcCow::Borrowed(_)));
        assert_eq!(arc_cow1, arc_cow2);
    }

    #[test]
    fn test_clone_owned() {
        let arc_cow1 = ArcCow::from(Arc::new(100));
        let arc_cow2 = arc_cow1.clone();
        
        // 克隆后仍为 Owned
        assert!(matches!(arc_cow1, ArcCow::Owned(_)));
        assert!(matches!(arc_cow2, ArcCow::Owned(_)));
        assert_eq!(arc_cow1, arc_cow2);
        
        // 修改原始值影响副本
        if let ArcCow::Owned(arc) = &arc_cow1 {
            // 类型是 Arc<T>，但我们需要可变引用来修改，所以需要强转
            let arc_mut = Arc::as_ptr(arc) as *mut i32;
            unsafe { *arc_mut = 200; } // 不安全操作，仅用于测试
        }
        assert_eq!(arc_cow1, arc_cow2);
    }

    #[test]
    fn test_comparison() {
        let borrowed1 = ArcCow::from("abc");
        let borrowed2 = ArcCow::from("abc");
        let owned1 = ArcCow::from("abc".to_string());
        let different = ArcCow::from("def");
        
        // 同值比较
        assert_eq!(borrowed1, borrowed2);
        assert_eq!(borrowed1, owned1);
        
        // 异值比较
        assert_ne!(borrowed1, different);
        
        // 排序
        assert!(borrowed1 < different);
        assert!(borrowed1 <= owned1);
        assert!(different > owned1);
    }

    #[test]
    fn test_hashing() {
        fn calculate_hash<T: Hash>(value: &T) -> u64 {
            let mut hasher = DefaultHasher::new();
            value.hash(&mut hasher);
            hasher.finish()
        }
        
        let borrowed_str :ArcCow<'static, str>= ArcCow::from("test");
        let owned_str = ArcCow::from("test".to_string());
        let different :ArcCow<'static, str>= ArcCow::from("different");
        
        // 相同内容不同变体应该具有相同哈希
        assert_eq!(calculate_hash(&borrowed_str), calculate_hash(&owned_str));
        
        // 不同内容应该不同哈希
        assert_ne!(calculate_hash(&borrowed_str), calculate_hash(&different));
    }

    #[test]
    fn test_cow_conversion() {
        let borrowed_cow = Cow::Borrowed("borrowed");
        let owned_cow :Cow<'static, str>= Cow::Owned("owned".to_string());
        
        let borrowed_arc_cow: ArcCow<'_, str> = borrowed_cow.into();
        let owned_arc_cow: ArcCow<'_, str> = owned_cow.into();
        
        assert!(matches!(borrowed_arc_cow, ArcCow::Borrowed(_)));
        assert!(matches!(owned_arc_cow, ArcCow::Owned(_)));
        assert_eq!(borrowed_arc_cow.as_ref(), "borrowed");
        assert_eq!(owned_arc_cow.as_ref(), "owned");
    }

    #[test]
    fn test_collection_handling() {
        let mut map = HashMap::new();
        
        map.insert(ArcCow::from("key1"), 1);
        map.insert(ArcCow::from("key2".to_string()), 2);
        // map.insert(ArcCow::from(Arc::new("key3")), 3);
        
        // 混合类型键查询
        assert_eq!(map.get(&ArcCow::from("key1")), Some(&1));
        assert_eq!(map.get(&ArcCow::from("key2")), Some(&2));
        // assert_eq!(map.get(&ArcCow::from("key3")), Some(&3));
        
        // 值查询
        assert_eq!(map.get(&ArcCow::from("key4")), None);
    }

    #[test]
    fn test_slice_conversions() {
        // Vec 转切片
        let vec = vec![1, 2, 3];
        let slice: ArcCow<'_, [i32]> = vec.into();
        assert_eq!(&*slice, &[1, 2, 3]);
        
        // &str 转 [u8]
        let bytes: ArcCow<'_, [u8]> = "text".into();
        assert_eq!(&*bytes, b"text");
    }

    #[test]
    fn test_debug_output() {
        let borrowed: ArcCow<'_, str> = "debug".into();
        let owned: ArcCow<'_, str> = "output".to_string().into();
        
        assert_eq!(format!("{:?}", borrowed), "\"debug\"");
        assert_eq!(format!("{:?}", owned), "\"output\"");
    }

    #[test]
    fn test_deref_and_as_ref() {
        let borrowed :ArcCow<'static, i32>= ArcCow::from(&42);
        let owned = ArcCow::from(Arc::new(100));
        
        // 解引用
        assert_eq!(*borrowed, 42);
        assert_eq!(*owned, 100);
        
        // as_ref
        let ref_borrowed: &i32 = borrowed.as_ref();
        let ref_owned: &i32 = owned.as_ref();
        assert_eq!(*ref_borrowed, 42);
        assert_eq!(*ref_owned, 100);
    }

    

    #[test]
    fn test_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        
        assert_send::<ArcCow<'static, str>>();
        assert_sync::<ArcCow<'static, str>>();
    }

    #[test]
    fn test_thread_safety() {
        let shared = ArcCow::from(Arc::new(100));
        
        let handle = std::thread::spawn(move || {
            assert_eq!(*shared, 100);
        });
        
        handle.join().unwrap();
    }
}



// ...existing code...
#[cfg(test)]
mod tests2 {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_deref_and_as_ref() {
        let borrowed_str = "hello";
        let arc_cow_borrowed = ArcCow::Borrowed(borrowed_str);
        assert_eq!(arc_cow_borrowed.as_ref(), "hello");
        assert_eq!(arc_cow_borrowed.as_ref(), "hello");

        let owned_string: Arc<str> = Arc::from("world");
        let arc_cow_owned = ArcCow::Owned(owned_string);
        assert_eq!(arc_cow_owned.as_ref(), "world");
        assert_eq!(arc_cow_owned.as_ref(), "world");
    }

    #[test]
    fn test_equality() {
        let s1 = "test";
        let s2: Arc<str> = Arc::from("test");
        let s3: Arc<str> = Arc::from("different");

        let borrowed = ArcCow::Borrowed(s1);
        let owned1 = ArcCow::Owned(s2.clone());
        let owned2 = ArcCow::Owned(s3.clone());

        assert_eq!(borrowed, owned1);
        assert_ne!(borrowed, owned2);
        assert_eq!(ArcCow::from(&s2), owned1);
    }

    #[test]
    fn test_ordering() {
        let s1 = "apple";
        let s2: Arc<str> = Arc::from("banana");
        let s3 = "cherry";

        let borrowed1 = ArcCow::Borrowed(s1);
        let owned = ArcCow::Owned(s2);
        let borrowed2 = ArcCow::Borrowed(s3);

        assert_eq!(borrowed1.cmp(&owned), Ordering::Less);
        assert_eq!(owned.cmp(&borrowed1), Ordering::Greater);
        assert_eq!(borrowed2.cmp(&owned), Ordering::Greater);
        assert_eq!(borrowed1.cmp(&borrowed1), Ordering::Equal);
    }

    #[test]
    fn test_hash() {
        let mut set = HashSet::new();
        let s1 = "hashable";
        let s2: Arc<str> = Arc::from("hashable");

        let borrowed = ArcCow::Borrowed(s1);
        let owned = ArcCow::Owned(s2);

        set.insert(borrowed);
        assert!(set.contains(&owned));
        assert_eq!(set.len(), 1);
    }

    #[test]
    fn test_clone() {
        // Clone Borrowed
        let borrowed_str = "borrowed";
        let arc_cow_borrowed = ArcCow::Borrowed(borrowed_str);
        let cloned_borrowed = arc_cow_borrowed.clone();
        if let ArcCow::Borrowed(inner) = cloned_borrowed {
            assert_eq!(inner, "borrowed");
        } else {
            panic!("Cloned Borrowed should be Borrowed");
        }
        // Ensure original is not moved
        assert_eq!(arc_cow_borrowed.as_ref(), "borrowed");

        // Clone Owned
        let owned_arc: Arc<str> = Arc::from("owned");
        let arc_cow_owned = ArcCow::Owned(owned_arc);
        // assert_eq!(Arc::strong_count(arc_cow_owned.clone().into_arc()), 2);
        // assert_eq!(Arc::strong_count(&arc_cow_owned.into_arc()), 1);
    }

    #[test]
    fn test_from_implementations() {
        // From<&'a T>
        let s_ref = "hello";
        let cow_from_ref :ArcCow<'static, str>= ArcCow::from(s_ref);
        assert_eq!(&*cow_from_ref, "hello");
        assert!(matches!(cow_from_ref, ArcCow::Borrowed(_)));

        // From<Arc<T>>
        let s_arc: Arc<str> = Arc::from("world");
        let cow_from_arc = ArcCow::from(s_arc.clone());
        assert_eq!(&*cow_from_arc, "world");
        assert!(matches!(cow_from_arc, ArcCow::Owned(_)));
        assert_eq!(Arc::strong_count(&s_arc), 2);

        // From<String> for ArcCow<'_, str>
        let string = String::from("string");
        let cow_from_string = ArcCow::from(string);
        assert_eq!(&*cow_from_string, "string");
        assert!(matches!(cow_from_string, ArcCow::Owned(_)));

        // From<Cow<'a, str>>
        let borrowed_cow: Cow<str> = Cow::Borrowed("cow_borrowed");
        let arc_cow_from_borrowed_cow = ArcCow::from(borrowed_cow);
        assert_eq!(&*arc_cow_from_borrowed_cow, "cow_borrowed");
        assert!(matches!(arc_cow_from_borrowed_cow, ArcCow::Borrowed(_)));

        let owned_cow: Cow<str> = Cow::Owned(String::from("cow_owned"));
        let arc_cow_from_owned_cow = ArcCow::from(owned_cow);
        assert_eq!(&*arc_cow_from_owned_cow, "cow_owned");
        assert!(matches!(arc_cow_from_owned_cow, ArcCow::Owned(_)));

        // From<Vec<T>> for ArcCow<'_, [T]>
        let vec = vec![1, 2, 3];
        let cow_from_vec = ArcCow::from(vec);
        assert_eq!(&*cow_from_vec, &[1, 2, 3]);
        assert!(matches!(cow_from_vec, ArcCow::Owned(_)));
    }

    #[test]
    fn test_debug_format() {
        let borrowed = ArcCow::Borrowed("debug");
        assert_eq!(format!("{:?}", borrowed), "\"debug\"");

        let owned: ArcCow<str> = ArcCow::Owned(Arc::from("debug"));
        assert_eq!(format!("{:?}", owned), "\"debug\"");
    }

    // Helper extension trait for tests
    trait ArcCowExt<T: ?Sized> {
        fn into_arc(self) -> Arc<T>;
    }

    impl<T: ?Sized + Clone> ArcCowExt<T> for ArcCow<'_, T>
    where
        T: ToOwned<Owned = T>,
    {
        fn into_arc(self) -> Arc<T> {
            match self {
                ArcCow::Borrowed(b) => Arc::new(b.to_owned()),
                ArcCow::Owned(o) => o,
            }
        }
    }
}