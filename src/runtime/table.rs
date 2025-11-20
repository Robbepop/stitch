use {
    crate::{
        decode::{Decode, DecodeError, Decoder},
        downcast::{DowncastMut, DowncastRef},
        elem::{Elem, ElemEntity, TypedElemEntity},
        guarded::Guarded,
        ref_::{ExternRef, FuncRef, Ref, RefType, RefTypeOf},
        store::{Handle, HandlePair, Store, StoreGuard, UnguardedHandle},
        trap::Trap,
    },
    std::{error::Error, fmt},
};

/// A WebAssembly table.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(transparent)]
pub struct Table(pub(crate) Handle<TableEntity>);

impl Table {
    /// Creates a new [`Table`] with the following parameters:
    ///
    /// * `store` - the [`Store`] in which to create the new [`Table`].
    /// * `ty` - the [`TableType`] of the new [`Table`].
    /// * `init_val` - the initial value of the new [`Table`]'s elements.
    ///
    /// # Errors
    ///
    /// If the [`RefType`] of `init_val` does not match that of the new [`Table`].
    ///
    /// # Panics
    ///
    /// * If `ty` is invalid.
    /// * If `init_val` does not originate from `store`.
    pub fn new(store: &mut Store, ty: TableType, init_val: Ref) -> Result<Self, TableError> {
        assert!(ty.is_valid());
        if init_val.type_() != ty.element() {
            return Err(TableError::TypeMismatch);
        }
        let table = match init_val {
            Ref::FuncRef(init_val) => TableEntity::FuncRef(TypedTableEntity::new(
                init_val,
                ty.minimum(),
                ty.maximum(),
                store.guard(),
            )),
            Ref::ExternRef(init_val) => TableEntity::ExternRef(TypedTableEntity::new(
                init_val,
                ty.minimum(),
                ty.maximum(),
                store.guard(),
            )),
        };
        Ok(Self(store.insert_table(table)))
    }

    /// Returns the [`TableType`] of this [`Table`] in the given `store`.
    ///
    /// # Panics
    ///
    /// If this [`Table`] does not originate from `store`.
    pub fn ty(self, store: &Store) -> TableType {
        match self.0.as_ref(store) {
            TableEntity::FuncRef(table) => table.ty(),
            TableEntity::ExternRef(table) => table.ty(),
        }
    }

    /// Returns the current value of this [`Table`]s `idx`-th element in the given `store.`
    ///
    /// # Errors
    ///
    /// If `idx` is out of bounds.
    ///
    /// # Panics
    ///
    /// If this [`Table`] does not originate from [`Store`].
    pub fn get(self, store: &Store, idx: u32) -> Option<Ref> {
        match self.0.as_ref(store) {
            TableEntity::FuncRef(table) => table.get(idx).map(Into::into),
            TableEntity::ExternRef(table) => table.get(idx).map(Into::into),
        }
    }

    /// Sets the value of this [`Table`]'s `idx`-th element to `new_val` in the given `store`.
    ///
    /// # Errors
    ///
    /// * If `idx` is out of bounds.
    /// * If the [`RefType`] of `new_val` does not match that of this [`Table`]s elements.
    ///
    /// # Panics
    ///
    /// * If this [`Table`] does not originate from `store`.
    /// * If `new_val` does not originate from `store`.
    pub fn set(self, store: &mut Store, idx: u32, new_val: Ref) -> Result<(), TableError> {
        match (self.0.as_mut(store), new_val) {
            (TableEntity::FuncRef(table), Ref::FuncRef(val)) => table.set(idx, val),
            (TableEntity::ExternRef(table), Ref::ExternRef(val)) => table.set(idx, val),
            _ => Err(TableError::TypeMismatch),
        }
    }

    /// Returns this [`Table`]'s current size.
    pub fn size(&self, store: &Store) -> u32 {
        match self.0.as_ref(store) {
            TableEntity::FuncRef(table) => table.size(),
            TableEntity::ExternRef(table) => table.size(),
        }
    }

    /// Grows this [`Table`] by `num` elements with initial value `init_val`.
    ///
    /// Returns the previous size of this [`Table`].
    ///
    /// # Errors
    ///
    /// - If the [`RefType`] of `init_val` does not match that of this [`Table`]'s elements.
    /// - If this [`Table`] failed to grow.
    ///
    /// # Panics
    ///
    /// - If `init_val` does not originate from `store`.
    pub fn grow(self, store: &mut Store, init_val: Ref, num: u32) -> Result<(), TableError> {
        match (self.0.as_mut(store), init_val) {
            (TableEntity::FuncRef(table), Ref::FuncRef(val)) => table.grow(val, num),
            (TableEntity::ExternRef(table), Ref::ExternRef(val)) => table.grow(val, num),
            _ => Err(TableError::TypeMismatch),
        }
        .map(|_| ())
    }

    pub(crate) fn init(
        self,
        store: &mut Store,
        dst_idx: u32,
        src_elem: Elem,
        src_idx: u32,
        num: u32,
    ) -> Result<(), Trap> {
        let (dst_table, src_elem) = HandlePair(self.0, src_elem.0).as_mut_pair(store);
        match (dst_table, src_elem) {
            (TableEntity::FuncRef(table), ElemEntity::FuncRef(src_elem)) => {
                table.init(dst_idx, src_elem, src_idx, num)
            }
            (TableEntity::ExternRef(table), ElemEntity::ExternRef(src_elem)) => {
                table.init(dst_idx, src_elem, src_idx, num)
            }
            _ => panic!(),
        }
    }
}

impl Guarded for Table {
    type Unguarded = UnguardedTable;
    type Guard = StoreGuard;

    unsafe fn from_unguarded(unguarded: UnguardedTable, guard: Self::Guard) -> Self {
        Self(Handle::from_unguarded(unguarded, guard))
    }

    fn to_unguarded(self, guard: Self::Guard) -> Self::Unguarded {
        self.0.to_unguarded(guard)
    }
}

pub(crate) type UnguardedTable = UnguardedHandle<TableEntity>;

/// The type of a [`Table`].
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct TableType {
    pub element: RefType,
    pub minimum: u32,
    pub maximum: Option<u32>,
}

impl TableType {
    /// Creates a new [`TableType`] with the following parameters:
    ///
    /// * `element` - The [`RefType`] of the [`Table`]'s elements.
    /// * `minimum` - The [`Table`]'s minimum size.
    /// * `maximum` - The [`Table`]'s maximum size, if any.
    pub fn new(element: RefType, minimum: u32, maximum: Option<u32>) -> Self {
        Self { element, minimum, maximum }
    }

    /// Returns the [`RefType`] of the [`Table`]'s elements.
    pub fn element(&self) -> RefType {
        self.element
    }

    /// Returns the [`Table`]'s minimum size.
    pub fn minimum(&self) -> u32 {
        self.minimum
    }

    /// Returns the [`Table`]'s maximum size, if any.
    pub fn maximum(&self) -> Option<u32> {
        self.maximum
    }

    pub(crate) fn is_valid(self) -> bool {
        if let Some(maximum) = self.maximum {
            if self.minimum > maximum {
                return false;
            }
        }
        true
    }

    pub(crate) fn is_subtype_of(self, other: Self) -> bool {
        if self.element != other.element {
            return false;
        }
        if self.minimum < other.minimum {
            return false;
        }
        match (self.maximum, other.maximum) {
            (None, Some(_)) => return false,
            (Some(maximum), Some(other_maximum)) if maximum > other_maximum => return false,
            _ => ()
        }
        true
    }
}

impl Decode for TableType {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        let element = decoder.decode()?;
        let has_maximum = match decoder.read_byte()? {
            0x00 => false,
            0x01 => true,
            _ => return Err(DecodeError::new("invalid table type")),
        };
        Ok(Self {
            element,
            minimum: decoder.decode()?,
            maximum: if has_maximum {
                Some(decoder.decode()?)
            } else {
                None
            },
        })
    }
}

/// An error returned by operations on a [`Table`].
#[derive(Debug)]
#[non_exhaustive]
pub enum TableError {
    /// The index is out of bounds.
    IdxOutOfBounds,
    /// The [`RefType`] does not match that of the [`Table`]'s elements.
    TypeMismatch,
    /// The [`Table`] failed to grow.
    FailedToGrow,
}

impl fmt::Display for TableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IdxOutOfBounds => write!(f, "index out of bounds"),
            Self::TypeMismatch => write!(f, "type does not match that of table's elements"),
            Self::FailedToGrow => write!(f, "table failed to grow"),
        }
    }
}

impl Error for TableError {}

#[derive(Debug)]
pub(crate) enum TableEntity {
    FuncRef(TypedTableEntity<FuncRef>),
    ExternRef(TypedTableEntity<ExternRef>),
}

impl TableEntity {
    pub(crate) fn downcast_ref<T>(&self) -> Option<&TypedTableEntity<T>>
    where
        T: Guarded,
        TypedTableEntity<T>: DowncastRef<Self>,
    {
        TypedTableEntity::downcast_ref(self)
    }

    pub(crate) fn downcast_mut<T>(&mut self) -> Option<&mut TypedTableEntity<T>>
    where
        T: Guarded,
        TypedTableEntity<T>: DowncastMut<Self>,
    {
        TypedTableEntity::downcast_mut(self)
    }
}

/// A typed [`TableEntity`].
#[derive(Debug)]
pub(crate) struct TypedTableEntity<T>
where
    T: Guarded,
{
    elements: Vec<T::Unguarded>,
    maximum: Option<u32>,
    guard: T::Guard,
}

impl<T> TypedTableEntity<T>
where
    T: Guarded,
{
    fn new(val: T, minimum: u32, maximum: Option<u32>, guard: T::Guard) -> Self {
        let val = val.to_unguarded(guard);
        unsafe { Self::new_unguarded(val, minimum, maximum, guard) }
    }

    unsafe fn new_unguarded(
        val: T::Unguarded,
        minimum: u32,
        maximum: Option<u32>,
        guard: T::Guard,
    ) -> Self {
        Self {
            elements: vec![val; minimum as usize],
            maximum,
            guard,
        }
    }

    fn get(&self, idx: u32) -> Option<T> {
        let val = self.get_unguarded(idx)?;
        Some(unsafe { T::from_unguarded(val, self.guard) })
    }

    pub(crate) fn get_unguarded(&self, idx: u32) -> Option<T::Unguarded> {
        let idx = idx as usize;
        let elem = self.elements.get(idx)?;
        Some(*elem)
    }

    fn set(&mut self, idx: u32, val: T) -> Result<(), TableError> {
        let val = val.to_unguarded(self.guard);
        unsafe { self.set_unguarded(idx, val) }
    }

    pub(crate) unsafe fn set_unguarded(
        &mut self,
        idx: u32,
        val: T::Unguarded,
    ) -> Result<(), TableError> {
        let elem = self
            .elements
            .get_mut(idx as usize)
            .ok_or(TableError::IdxOutOfBounds)?;
        *elem = val;
        Ok(())
    }

    pub(crate) fn size(&self) -> u32 {
        self.elements.len() as u32
    }

    fn grow(&mut self, val: T, num: u32) -> Result<u32, TableError> {
        let val = val.to_unguarded(self.guard);
        unsafe { self.grow_unguarded(val, num) }
    }

    pub(crate) unsafe fn grow_unguarded(
        &mut self,
        val: T::Unguarded,
        num: u32,
    ) -> Result<u32, TableError> {
        if num > self.maximum.unwrap_or(u32::MAX) - self.size() {
            return Err(TableError::FailedToGrow)?;
        }
        let num = num as usize;
        let size = self.size();
        self.elements.resize(self.elements.len() + num, val);
        Ok(size)
    }

    pub(crate) unsafe fn fill_unguarded(
        &mut self,
        idx: u32,
        val: T::Unguarded,
        num: u32,
    ) -> Result<(), Trap> {
        let elems = self
            .elements
            .get_mut(idx as usize..)
            .and_then(|elems| elems.get_mut(..num as usize))
            .ok_or(Trap::TableAccessOutOfBounds)?;
        elems.fill(val);
        Ok(())
    }

    pub(crate) fn copy(
        &mut self,
        dst_idx: u32,
        src_table: &TypedTableEntity<T>,
        src_idx: u32,
        num: u32,
    ) -> Result<(), Trap> {
        let dst_elems = self
            .elements
            .get_mut(dst_idx as usize..)
            .and_then(|elems| elems.get_mut(..num as usize))
            .ok_or(Trap::TableAccessOutOfBounds)?;
        let src_elems = src_table
            .elements
            .get(src_idx as usize..)
            .and_then(|elems| elems.get(..num as usize))
            .ok_or(Trap::TableAccessOutOfBounds)?;
        dst_elems.copy_from_slice(src_elems);
        Ok(())
    }

    pub(crate) fn copy_within(&mut self, dst_idx: u32, src_idx: u32, num: u32) -> Result<(), Trap> {
        if num > self.size() || dst_idx > self.size() - num || src_idx > self.size() - num {
            return Err(Trap::TableAccessOutOfBounds)?;
        }
        self.elements.copy_within(
            src_idx as usize..src_idx as usize + num as usize,
            dst_idx as usize,
        );
        Ok(())
    }

    pub(crate) fn init(
        &mut self,
        dst_idx: u32,
        src_elem: &TypedElemEntity<T>,
        src_idx: u32,
        num: u32,
    ) -> Result<(), Trap> {
        let dst_elems = self
            .elements
            .get_mut(dst_idx as usize..)
            .and_then(|elems| elems.get_mut(..num as usize))
            .ok_or(Trap::TableAccessOutOfBounds)?;
        let src_elems = src_elem
            .elems()
            .get(src_idx as usize..)
            .and_then(|elems| elems.get(..num as usize))
            .ok_or(Trap::TableAccessOutOfBounds)?;
        dst_elems.copy_from_slice(src_elems);
        Ok(())
    }
}

impl<T> TypedTableEntity<T>
where
    T: Guarded + RefTypeOf,
{
    fn ty(&self) -> TableType {
        TableType::new(T::ref_type_of(), self.size(), self.maximum)
    }
}

impl DowncastRef<TableEntity> for TypedTableEntity<FuncRef> {
    fn downcast_ref(table: &TableEntity) -> Option<&Self> {
        if let TableEntity::FuncRef(table) = table {
            Some(table)
        } else {
            None
        }
    }
}

impl DowncastMut<TableEntity> for TypedTableEntity<FuncRef> {
    fn downcast_mut(table: &mut TableEntity) -> Option<&mut Self> {
        if let TableEntity::FuncRef(table) = table {
            Some(table)
        } else {
            None
        }
    }
}

impl DowncastRef<TableEntity> for TypedTableEntity<ExternRef> {
    fn downcast_ref(table: &TableEntity) -> Option<&Self> {
        if let TableEntity::ExternRef(table) = table {
            Some(table)
        } else {
            None
        }
    }
}

impl DowncastMut<TableEntity> for TypedTableEntity<ExternRef> {
    fn downcast_mut(table: &mut TableEntity) -> Option<&mut Self> {
        if let TableEntity::ExternRef(table) = table {
            Some(table)
        } else {
            None
        }
    }
}
