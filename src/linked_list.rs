// Copyright (c) 2023 Huawei Device Co., Ltd.
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use core::fmt::{Debug, Formatter};
use core::marker::PhantomData;
use core::ptr::null;

// todo: Considers deleting PhantomData.

/// Linked list implementation, provides two sets of methods for getting nodes and members.
/// Only tail insertion, reading, and eject are supported.
pub(crate) struct LinkedList<T> {
    head: *const Node<T>,
    tail: *const Node<T>,
    len: usize,
    marker: PhantomData<Box<Node<T>>>,
}

impl<T> LinkedList<T> {
    /// Creates LinkedList.
    pub(crate) const fn new() -> Self {
        LinkedList {
            head: null(),
            tail: null(),
            len: 0,
            marker: PhantomData,
        }
    }

    /// Gets length of the list.
    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.len
    }

    /// Determines whether the linked list is empty.
    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Inserts an element at the end of the list
    pub(crate) fn push_back(&mut self, value: T) {
        let mut node = Box::new(Node::new(value));
        unsafe {
            // Sets prev to LinkedList.tail
            node.prev = self.tail;
            // Gets an internal element pointer.
            let node = Box::leak(node) as *const Node<T>;

            if self.tail.is_null() {
                self.head = node;
            } else {
                (*(self.tail as *mut Node<T>)).next = node;
            }

            self.tail = node;
            self.len += 1;
        }
    }

    /// Pops an element from the end of the list.
    pub(crate) fn pop_back(&mut self) -> Option<T> {
        if self.tail.is_null() {
            None
        } else {
            unsafe {
                let node = Box::from_raw(self.tail as *mut Node<T>);
                self.tail = node.prev;

                if self.tail.is_null() {
                    self.head = null();
                } else {
                    (*(self.tail as *mut Node<T>)).next = null();
                }

                self.len -= 1;
                Some(node.into_element())
            }
        }
    }

    /// Gets an ordinary iterator for a linked list.
    #[inline]
    pub(crate) fn iter(&self) -> Iter<'_, T> {
        Iter {
            head: self.head,
            tail: self.tail,
            len: self.len,
            marker: PhantomData,
        }
    }

    /// Gets a mutable iterator for the linked list.
    #[inline]
    pub(crate) fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            head: self.head,
            tail: self.tail,
            len: self.len,
            marker: PhantomData,
        }
    }

    /// Gets the normal cursor of the list and sets the starting point to the list header.
    #[inline]
    pub(crate) fn cursor_front(&self) -> Cursor<'_, T> {
        Cursor {
            index: 0,
            current: self.head,
            list: self,
        }
    }

    /// Gets the variable cursor of the list and sets the starting point to the list header.
    #[inline]
    pub(crate) fn cursor_front_mut(&mut self) -> CursorMut<'_, T> {
        CursorMut {
            index: 0,
            current: self.head,
            list: self,
        }
    }

    /// Gets the normal cursor of the list and sets the starting point to the end of the list.
    #[cfg(feature = "list_array")]
    #[inline]
    pub(crate) fn cursor_back(&self) -> Cursor<'_, T> {
        Cursor {
            index: self.len.saturating_sub(1),
            current: self.tail,
            list: self,
        }
    }

    /// Gets the variable cursor of the list and sets the start to the end of the list.
    #[cfg(feature = "list_array")]
    #[inline]
    pub(crate) fn cursor_back_mut(&mut self) -> CursorMut<'_, T> {
        CursorMut {
            index: self.len.saturating_sub(1),
            current: self.tail,
            list: self,
        }
    }

    /// Gets a mutable reference to the tail element of the list.
    #[cfg(feature = "list_array")]
    #[inline]
    pub(crate) fn back(&self) -> Option<&T> {
        if self.tail.is_null() {
            None
        } else {
            unsafe { Some(&(*self.tail).element) }
        }
    }

    /// Gets a mutable reference to the tail element of the list.
    #[inline]
    pub(crate) fn back_mut(&mut self) -> Option<&mut T> {
        if self.tail.is_null() {
            None
        } else {
            unsafe { Some(&mut (*(self.tail as *mut Node<T>)).element) }
        }
    }

    /// Gets a common reference to the node at the end of the linked list.
    #[cfg(feature = "list_array")]
    #[inline]
    pub(crate) fn back_node(&self) -> Option<&Node<T>> {
        if self.tail.is_null() {
            None
        } else {
            unsafe { Some(&(*self.tail)) }
        }
    }

    /// Gets a mutable reference to the node at the end of the linked list.
    #[cfg(feature = "list_array")]
    #[inline]
    pub(crate) fn back_node_mut(&mut self) -> Option<&mut Node<T>> {
        if self.tail.is_null() {
            None
        } else {
            unsafe {
                // Sets node.parent to the current linked_list in order to delete node.
                let node = &mut *(self.tail as *mut Node<T>);
                node.parent = self as *const LinkedList<T>;
                Some(node)
            }
        }
    }

    /// Removes a node from the linked list.
    pub(crate) unsafe fn unlink_node(&mut self, node: *const Node<T>) {
        let node = &mut (*(node as *mut Node<T>));

        if node.prev.is_null() {
            self.head = node.next;
        } else {
            (*(node.prev as *mut Node<T>)).next = node.next;
        }

        if node.next.is_null() {
            self.tail = node.prev;
        } else {
            (*(node.next as *mut Node<T>)).prev = node.prev;
        }

        self.len -= 1;
    }
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Debug> Debug for LinkedList<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        for (n, item) in self.iter().enumerate() {
            if n != 0 {
                write!(f, ",")?;
            }
            write!(f, "{item:?}")?;
        }
        Ok(())
    }
}

impl<T: PartialEq> PartialEq for LinkedList<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }
        for (a, b) in self.iter().zip(other.iter()) {
            if a.ne(b) {
                return false;
            }
        }
        true
    }
}

impl<T: Clone> Clone for LinkedList<T> {
    fn clone(&self) -> Self {
        let mut new_list = LinkedList::new();
        for item in self.iter() {
            new_list.push_back(item.clone());
        }
        new_list
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        while self.len != 0 {
            let _ = self.pop_back();
        }
    }
}

// We need to use static to store the JsonValue, so we need to make the LinkedList implement Send and Sync.
// However, when using this list, locking is still required under concurrent conditions.
unsafe impl<T: Send> Send for LinkedList<T> {}
unsafe impl<T: Sync> Sync for LinkedList<T> {}

/// Linked list node, only through a linked list cursor to get the node.
pub struct Node<T> {
    next: *const Node<T>,
    prev: *const Node<T>,
    parent: *const LinkedList<T>,
    element: T,
}

impl<T> Node<T> {
    /// Creates a linked list node.
    pub(crate) fn new(element: T) -> Self {
        Node {
            next: null(),
            prev: null(),
            parent: null(),
            element,
        }
    }

    /// Retrieves the member inside the list node.
    pub(crate) fn into_element(self) -> T {
        self.element
    }

    /// Gets a common reference to an internal member of a linked list node.
    pub(crate) fn get_element_mut(&mut self) -> &mut T {
        &mut self.element
    }

    /// Removes the node itself from the linked list and returns the member below.
    #[cfg(feature = "c_adapter")]
    pub(crate) fn remove_self(&mut self) -> Option<T> {
        let list = unsafe { &mut *(self.parent as *mut LinkedList<T>) };
        let mut cursor = CursorMut {
            index: 0,
            current: self as *const Node<T>,
            list,
        };
        cursor.remove_current()
    }
}

/// A common iterator of a linked list.
pub struct Iter<'a, T: 'a> {
    head: *const Node<T>,
    tail: *const Node<T>,
    len: usize,
    marker: PhantomData<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        if self.len == 0 || self.head.is_null() {
            None
        } else {
            let node = unsafe { &*(self.head as *mut Node<T>) };
            self.len -= 1;
            self.head = node.next;
            Some(&node.element)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // Returns a tuple representing the remaining range of iterators.
        (self.len, Some(self.len))
    }

    #[inline]
    fn last(mut self) -> Option<&'a T> {
        // Uses the iterator traversal and returns the last element.
        self.next_back()
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<&'a T> {
        if self.len == 0 || self.tail.is_null() {
            None
        } else {
            let node = unsafe { &*(self.tail as *mut Node<T>) };
            self.len -= 1;
            self.tail = node.prev;
            Some(&node.element)
        }
    }
}

/// A variable iterator of a linked list.
pub struct IterMut<'a, T: 'a> {
    head: *const Node<T>,
    tail: *const Node<T>,
    len: usize,
    marker: PhantomData<&'a mut Node<T>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    #[inline]
    fn next(&mut self) -> Option<&'a mut T> {
        if self.len == 0 || self.head.is_null() {
            None
        } else {
            let node = unsafe { &mut *(self.head as *mut Node<T>) };
            self.len -= 1;
            self.head = node.next;
            Some(&mut node.element)
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // Returns a tuple representing the remaining range of iterators.
        (self.len, Some(self.len))
    }

    #[inline]
    fn last(mut self) -> Option<&'a mut T> {
        // Uses the iterator traversal and returns the last element.
        self.next_back()
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a mut T> {
        if self.len == 0 || self.tail.is_null() {
            None
        } else {
            let node = unsafe { &mut *(self.tail as *mut Node<T>) };
            self.len -= 1;
            self.tail = node.prev;
            Some(&mut node.element)
        }
    }
}

/// A common cursor for a linked list. When the list is empty,
/// it points to a virtual location (pointing to a node that does not actually exist).
pub(crate) struct Cursor<'a, T: 'a> {
    index: usize,
    current: *const Node<T>,
    list: &'a LinkedList<T>,
}

impl<'a, T> Cursor<'a, T> {
    /// Gets the position the cursor is pointing to.
    /// If the cursor points to a virtual position, return None.
    #[inline]
    pub(crate) fn index(&self) -> Option<usize> {
        if self.current.is_null() {
            return None;
        }
        Some(self.index)
    }

    /// The cursor moves back.
    #[inline]
    pub(crate) fn move_next(&mut self) {
        if self.current.is_null() {
            self.current = self.list.head;
            self.index = 0;
        } else {
            self.current = unsafe { (*self.current).next };
            self.index += 1;
        }
    }

    /// The cursor moves forward.
    #[cfg(feature = "list_array")]
    #[inline]
    pub(crate) fn move_prev(&mut self) {
        if self.current.is_null() {
            self.current = self.list.tail;
            self.index = self.list.len().saturating_sub(1);
        } else {
            self.current = unsafe { (*self.current).prev };
            self.index = self.index.checked_sub(1).unwrap_or_else(|| self.list.len());
        }
    }

    /// Gets the cursor.
    #[inline]
    pub(crate) fn current(&self) -> Option<&'a T> {
        if self.current.is_null() {
            None
        } else {
            unsafe { Some(&(*self.current).element) }
        }
    }

    /// Gets a reference to the current node.
    #[inline]
    pub(crate) fn current_node(&self) -> Option<&'a Node<T>> {
        if self.current.is_null() {
            None
        } else {
            unsafe { Some(&*(self.current)) }
        }
    }

    #[cfg(feature = "list_object")]
    #[inline]
    pub(crate) fn current_node_ptr(&self) -> *const Node<T> {
        self.current
    }
}

pub(crate) struct CursorMut<'a, T: 'a> {
    index: usize,
    current: *const Node<T>,
    list: &'a mut LinkedList<T>,
}

impl<'a, T> CursorMut<'a, T> {
    /// Gets the index.
    #[inline]
    pub(crate) fn index(&self) -> Option<usize> {
        if self.current.is_null() {
            return None;
        }
        Some(self.index)
    }

    /// The cursor moves beck.
    #[inline]
    pub(crate) fn move_next(&mut self) {
        if self.current.is_null() {
            self.current = self.list.head;
            self.index = 0;
        } else {
            self.current = unsafe { (*self.current).next };
            self.index += 1;
        }
    }

    /// The cursor moves forward.
    #[cfg(feature = "list_array")]
    #[inline]
    pub(crate) fn move_prev(&mut self) {
        if self.current.is_null() {
            self.current = self.list.tail;
            self.index = self.list.len().saturating_sub(1);
        } else {
            self.current = unsafe { (*self.current).prev };
            self.index = self.index.checked_sub(1).unwrap_or_else(|| self.list.len());
        }
    }

    /// Gets a mutable reference to the current element.
    #[cfg(feature = "list_object")]
    #[inline]
    pub(crate) fn current(&mut self) -> Option<&mut T> {
        if self.current.is_null() {
            None
        } else {
            unsafe { Some(&mut (*(self.current as *mut Node<T>)).element) }
        }
    }

    /// Gets a mutable reference to the current node.
    #[inline]
    pub(crate) fn current_node(&mut self) -> Option<&'a mut Node<T>> {
        if self.current.is_null() {
            None
        } else {
            unsafe {
                let node = &mut *(self.current as *mut Node<T>);
                node.parent = self.list as *mut LinkedList<T>;
                Some(node)
            }
        }
    }

    /// Deletes the node to which the cursor is pointing.
    #[inline]
    pub(crate) fn remove_current(&mut self) -> Option<T> {
        if self.current.is_null() {
            return None;
        }

        let unlinked_node = self.current;
        unsafe {
            self.current = (*unlinked_node).next;
            self.list.unlink_node(unlinked_node);
            let unlinked_node = Box::from_raw(unlinked_node as *mut Node<T>);
            Some(unlinked_node.element)
        }
    }
}

#[cfg(test)]
mod ut_linked_list {
    use crate::LinkedList;

    /// UT test for `LinkedList::pop_back`.
    ///
    /// # Title
    /// ut_linked_list_pop_back
    ///
    /// # Brief
    /// 1. Creates a `LinkedList`.
    /// 2. Calls `LinkedList::pop_back` on it.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_linked_list_pop_back() {
        let mut list = LinkedList::new();
        assert_eq!(list.pop_back(), None);

        list.push_back(1i32);
        assert_eq!(list.pop_back(), Some(1));
    }

    /// UT test for `LinkedList::iter_mut`.
    ///
    /// # Title
    /// ut_linked_list_iter_mut
    ///
    /// # Brief
    /// 1. Creates a `LinkedList`.
    /// 2. Calls `LinkedList::iter_mut` on it.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_linked_list_iter_mut() {
        let mut list = LinkedList::new();
        list.push_back(1i32);
        list.push_back(2i32);

        let mut iter = list.iter_mut();
        assert_eq!(iter.next(), Some(&mut 1));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), None);
    }

    /// UT test for `LinkedList::back`.
    ///
    /// # Title
    /// ut_linked_list_back
    ///
    /// # Brief
    /// 1. Creates a `LinkedList`.
    /// 2. Calls `LinkedList::back` on it.
    /// 3. Checks if the test results are correct.
    #[cfg(feature = "list_array")]
    #[test]
    fn ut_linked_list_back() {
        let mut list = LinkedList::new();
        assert_eq!(list.back(), None);

        list.push_back(1i32);
        assert_eq!(list.back(), Some(&1));
    }

    /// UT test for `LinkedList::back_mut`.
    ///
    /// # Title
    /// ut_linked_list_back_mut
    ///
    /// # Brief
    /// 1. Creates a `LinkedList`.
    /// 2. Calls `LinkedList::back_mut` on it.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_linked_list_back_mut() {
        let mut list = LinkedList::new();
        assert_eq!(list.back_mut(), None);

        list.push_back(1i32);
        assert_eq!(list.back_mut(), Some(&mut 1));
    }

    /// UT test for `LinkedList::back_node`.
    ///
    /// # Title
    /// ut_linked_list_back_node
    ///
    /// # Brief
    /// 1. Creates a `LinkedList`.
    /// 2. Calls `LinkedList::back_node` on it.
    /// 3. Checks if the test results are correct.
    #[cfg(feature = "list_array")]
    #[test]
    fn ut_linked_list_back_node() {
        let mut list = LinkedList::new();
        assert!(list.back_node().is_none());

        list.push_back(1i32);
        assert!(list.back_node().is_some());
    }

    /// UT test for `LinkedList::back_node_mut`.
    ///
    /// # Title
    /// ut_linked_list_back_node_mut
    ///
    /// # Brief
    /// 1. Creates a `LinkedList`.
    /// 2. Calls `LinkedList::back_node_mut` on it.
    /// 3. Checks if the test results are correct.
    #[cfg(feature = "list_array")]
    #[test]
    fn ut_linked_list_back_node_mut() {
        let mut list = LinkedList::new();
        assert!(list.back_node_mut().is_none());

        list.push_back(1i32);
        assert!(list.back_node_mut().is_some());
    }

    /// UT test for `LinkedList::default`.
    ///
    /// # Title
    /// ut_linked_list_default
    ///
    /// # Brief
    /// 1. Calls `LinkedList::default` to create a `LinkedList`.
    /// 2. Checks if the test results are correct.
    #[test]
    fn ut_linked_list_default() {
        assert_eq!(LinkedList::<i32>::default(), LinkedList::<i32>::new());
    }

    /// UT test for `LinkedList::eq`.
    ///
    /// # Title
    /// ut_linked_list_eq
    ///
    /// # Brief
    /// 1. Creates some `LinkedList`s.
    /// 2. Calls `LinkedList::eq` on them.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_linked_list_eq() {
        let mut list1 = LinkedList::new();
        list1.push_back(1i32);

        let mut list2 = LinkedList::new();
        list2.push_back(1i32);
        list2.push_back(2i32);

        let mut list3 = LinkedList::new();
        list3.push_back(1i32);
        list3.push_back(3i32);

        assert_eq!(list1, list1);
        assert_ne!(list1, list2);
        assert_ne!(list2, list3);
    }

    /// UT test for `LinkedList::clone`.
    ///
    /// # Title
    /// ut_linked_list_clone
    ///
    /// # Brief
    /// 1. Creates a `LinkedList`.
    /// 2. Calls `LinkedList::clone` to create a copy of it.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_linked_list_clone() {
        let mut list1 = LinkedList::new();
        list1.push_back(1i32);

        let list2 = list1.clone();
        assert_eq!(list1, list2);
    }

    /// UT test for `LinkedList::fmt`.
    ///
    /// # Title
    /// ut_linked_list_fmt
    ///
    /// # Brief
    /// 1. Creates a `LinkedList`.
    /// 2. Calls `LinkedList::fmt` on it.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_linked_list_fmt() {
        let mut list = LinkedList::new();
        list.push_back(1i32);
        list.push_back(2i32);
        assert_eq!(format!("{list:?}"), "1,2");
    }

    /// UT test for `Cursor::index`.
    ///
    /// # Title
    /// ut_cursor_index
    ///
    /// # Brief
    /// 1. Creates a `LinkedList` and a `Cursor`.
    /// 2. Calls `Cursor::index`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_cursor_index() {
        let mut list = LinkedList::new();
        list.push_back(1i32);

        let mut cursor = list.cursor_front();
        assert_eq!(cursor.index(), Some(0));

        cursor.move_next();
        assert_eq!(cursor.index(), None);
    }

    /// UT test for `Cursor::move_next`.
    ///
    /// # Title
    /// ut_cursor_move_next
    ///
    /// # Brief
    /// 1. Creates a `LinkedList` and a `Cursor`.
    /// 2. Calls `Cursor::move_next`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_cursor_move_next() {
        let mut list = LinkedList::new();
        list.push_back(1i32);
        list.push_back(2i32);

        let mut cursor = list.cursor_front();
        assert_eq!(cursor.current(), Some(&1));

        cursor.move_next();
        assert_eq!(cursor.current(), Some(&2));

        cursor.move_next();
        assert_eq!(cursor.current(), None);

        cursor.move_next();
        assert_eq!(cursor.current(), Some(&1));
    }

    /// UT test for `Cursor::move_prev`.
    ///
    /// # Title
    /// ut_cursor_move_prev
    ///
    /// # Brief
    /// 1. Creates a `LinkedList` and a `Cursor`.
    /// 2. Calls `Cursor::move_prev`.
    /// 3. Checks if the test results are correct.
    #[cfg(feature = "list_array")]
    #[test]
    fn ut_cursor_move_prev() {
        let mut list = LinkedList::new();
        list.push_back(1i32);
        list.push_back(2i32);

        let mut cursor = list.cursor_front();
        assert_eq!(cursor.current(), Some(&1));

        cursor.move_prev();
        assert_eq!(cursor.current(), None);

        cursor.move_prev();
        assert_eq!(cursor.current(), Some(&2));

        cursor.move_prev();
        assert_eq!(cursor.current(), Some(&1));
    }

    /// UT test for `Cursor::current_node`.
    ///
    /// # Title
    /// ut_cursor_current_node
    ///
    /// # Brief
    /// 1. Creates a `LinkedList` and a `Cursor`.
    /// 2. Calls `Cursor::current_node`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_cursor_current_node() {
        let mut list = LinkedList::new();
        list.push_back(1i32);

        let mut cursor = list.cursor_front();
        assert!(cursor.current_node().is_some());

        cursor.move_next();
        assert!(cursor.current_node().is_none());
    }

    /// UT test for `CursorMut::index`.
    ///
    /// # Title
    /// ut_cursor_mut_index
    ///
    /// # Brief
    /// 1. Creates a `LinkedList` and a `CursorMut`.
    /// 2. Calls `CursorMut::index`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_cursor_mut_index() {
        let mut list = LinkedList::new();
        list.push_back(1i32);

        let mut cursor = list.cursor_front_mut();
        assert_eq!(cursor.index(), Some(0));

        cursor.move_next();
        assert_eq!(cursor.index(), None);
    }

    /// UT test for `CursorMut::move_next`.
    ///
    /// # Title
    /// ut_cursor_mut_move_next
    ///
    /// # Brief
    /// 1. Creates a `LinkedList` and a `CursorMut`.
    /// 2. Calls `CursorMut::move_next`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_cursor_mut_move_next() {
        let mut list = LinkedList::new();
        list.push_back(1i32);

        let mut cursor = list.cursor_front_mut();
        assert!(cursor.current_node().is_some());

        cursor.move_next();
        assert!(cursor.current_node().is_none());

        cursor.move_next();
        assert!(cursor.current_node().is_some());
    }

    /// UT test for `CursorMut::move_prev`.
    ///
    /// # Title
    /// ut_cursor_mut_move_prev
    ///
    /// # Brief
    /// 1. Creates a `LinkedList` and a `CursorMut`.
    /// 2. Calls `CursorMut::move_prev`.
    /// 3. Checks if the test results are correct.
    #[cfg(feature = "list_array")]
    #[test]
    fn ut_cursor_mut_move_prev() {
        let mut list = LinkedList::new();
        list.push_back(1i32);

        let mut cursor = list.cursor_front_mut();
        assert!(cursor.current_node().is_some());

        cursor.move_prev();
        assert!(cursor.current_node().is_none());

        cursor.move_prev();
        assert!(cursor.current_node().is_some());
    }

    /// UT test for `CursorMut::current`.
    ///
    /// # Title
    /// ut_cursor_mut_current
    ///
    /// # Brief
    /// 1. Creates a `LinkedList` and a `CursorMut`.
    /// 2. Calls `CursorMut::current`.
    /// 3. Checks if the test results are correct.
    #[cfg(feature = "list_object")]
    #[test]
    fn ut_cursor_mut_current() {
        let mut list = LinkedList::new();
        list.push_back(1i32);

        let mut cursor = list.cursor_front_mut();
        assert_eq!(cursor.current(), Some(&mut 1));

        cursor.move_next();
        assert_eq!(cursor.current(), None);
    }

    /// UT test for `CursorMut::current`.
    ///
    /// # Title
    /// ut_cursor_mut_current
    ///
    /// # Brief
    /// 1. Creates a `LinkedList` and a `CursorMut`.
    /// 2. Calls `CursorMut::current`.
    /// 3. Checks if the test results are correct.
    #[test]
    fn ut_cursor_mut_remove_current() {
        let mut list = LinkedList::new();
        list.push_back(1i32);

        let mut cursor = list.cursor_front_mut();
        assert_eq!(cursor.remove_current(), Some(1));
        assert_eq!(cursor.remove_current(), None);
    }
}
