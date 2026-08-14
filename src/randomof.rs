use std::{collections::{HashMap, HashSet}, hash::Hash};

use crate::{DiceExt, DiceExtSides};

pub trait RandomOf<T> : Clone {
    type Output;
    fn random_of(&self) -> Self::Output;
}

pub trait PlainRandomOf : Clone {
    type Output;
    fn random_of() -> Self::Output;
}

pub trait KeyedRandomOf<K,T> : Clone {
    type Output;
    fn random_of(&self) -> Self::Output;
}

pub trait KeyedAllocatorRandomOf<K,T,A> : Clone {
    type Output;
    fn random_of(&self) -> Self::Output;
}

impl<T> RandomOf<T> for Vec<T>
where T: Clone
{
    type Output = T;
    /// Get a random item from some vector.
    /// 
    /// # Panic
    /// An empty `Vec` will cause a panic.
    fn random_of(&self) -> Self::Output {
        if self.is_empty() { panic!("Empty Vec - can't pick a random from that. Anyway… Ta-ta 'til that's fixed.")}
        T::clone(&self[1.d(self.len() as DiceExtSides)-1])
    }
}

impl<T> RandomOf<T> for HashSet<T>
where T: Clone
{
    type Output = T;
    /// Get a random item from some vector.
    /// 
    /// # Panic
    /// An empty `HashSet` will cause a panic.
    fn random_of(&self) -> Self::Output {
        if self.is_empty() { panic!("Empty HashSet - can't pick a random from that. Anyway… Ta-ta 'til that's fixed.")}
        let Some(ent) = self.iter().nth((1_usize.d(self.len() as DiceExtSides) - 1) as usize) else {
            panic!("For some reason the HashSet has less entries in it than .len() suggests?!");
        };
        T::clone(ent)
    }
}

impl <T> RandomOf<T> for HashMap<String, T>
where T: Clone
{
    type Output = T;
    fn random_of(&self) -> Self::Output {
        if self.is_empty() { panic!("Empty HashMap - can't pick a random from that. Anyway… Ta-ta 'til that's fixed.")}
        let Some((_,ent)) = self.iter().nth((1_usize.d(self.len() as DiceExtSides) - 1) as usize) else {
            panic!("For some reason the HashMap has less entries in it than .len() suggests?!");
        };
        T::clone(ent)
    }
}

impl <K,T> KeyedRandomOf<K,T> for HashMap<K,T>
where T: Clone, K: Hash + Clone
{
    type Output = T;
    fn random_of(&self) -> Self::Output {
        if self.is_empty() { panic!("Empty HashMap - can't pick a random from that. Anyway… Ta-ta 'til that's fixed.")}
        let Some((_,ent)) = self.iter().nth((1_usize.d(self.len() as DiceExtSides) - 1) as usize) else {
            panic!("For some reason the HashMap has less entries in it than .len() suggests?!");
        };
        T::clone(ent)
    }
}

impl <K,T,A> KeyedAllocatorRandomOf<K,T,A> for HashMap<K,T,A>
where
    T: Clone,
    K: Hash + Clone,
    A: Clone,
{
    type Output = T;
    fn random_of(&self) -> Self::Output {
        if self.is_empty() { panic!("Empty HashMap - can't pick a random from that. Anyway… Ta-ta 'til that's fixed.")}
        let Some((_,ent)) = self.iter().nth((1_usize.d(self.len() as DiceExtSides) - 1) as usize) else {
            panic!("For some reason the HashMap has less entries in it than .len() suggests?!");
        };
        T::clone(ent)
    }
}

impl <T, const N: usize> RandomOf<T> for [T;N]
where
    T: Clone,
{
    type Output = T;
    fn random_of(&self) -> Self::Output {
        if N == 0 { panic!("Empty array - can't pick a random from that!")}

        let idx = 1.d(N as DiceExtSides) - 1;
        self[idx].clone()
    }
}
