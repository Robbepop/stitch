use {
    crate::{
        data::{Data, DataEntity},
        decode::{Decode, DecodeError, Decoder},
        executor::SavedRegs,
        func::Context,
        guarded::Guarded,
        stack::Stack,
        store::{Handle, HandlePair, Store, StoreGuard, UnguardedHandle},
        trap::Trap,
    },
    std::{error::Error, fmt},
};

/// A WebAssembly memory.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct Memory(pub(crate) Handle<MemoryEntity>);

impl Memory {
    /// Creates a new [`Memory`] with the following parameters:
    /// 
    /// * `store` - the [`Store`] in which to create the new [`Memory`].
    /// * `ty` - the [`MemoryType`] of the new [`Memory`].
    ///
    /// # Panics
    ///
    /// * If `ty` is invalid.
    pub fn new(store: &mut Store, ty: MemoryType) -> Self {
        assert!(ty.is_valid(), "invalid memory type");
        Self(store.insert_memory(MemoryEntity::new(ty.minimum, ty.maximum)))
    }

    /// Returns the [`MemoryType`] of this [`Memory`]
    pub fn type_(self, store: &Store) -> MemoryType {
        self.0.as_ref(store).ty()
    }

    /// Returns this [`Memory`]'s bytes as a slice.
    pub fn bytes(self, store: &Store) -> &[u8] {
        self.0.as_ref(store).bytes()
    }

    /// Returns this [`Memory`]'s bytes as a mutable slice.
    pub fn bytes_mut(self, store: &mut Store) -> &mut [u8] {
        self.0.as_mut(store).bytes_mut()
    }

    /// Returns this [`Memory`]'s current size.
    pub fn size(&self, store: &Store) -> u32 {
        self.0.as_ref(store).size()
    }

    /// Grows this [`Memory`] by `num` pages.
    ///
    /// Returns the previous size of this [`Memory`].
    /// 
    /// # Errors
    ///
    /// If this [`Memory`] failed to grow.
    pub fn grow(self, mut context: impl Context, num: u32) -> Result<u32, MemoryError> {
        let (store, stack) = context.into_parts();
        self.0.as_mut(store).grow(stack, num)
    }

    pub(crate) fn init(
        self,
        store: &mut Store,
        dst_offset: u32,
        src_data: Data,
        src_offset: u32,
        count: u32,
    ) -> Result<(), Trap> {
        let (dst_table, src_data) = HandlePair(self.0, src_data.0).as_mut_pair(store);
        dst_table.init(dst_offset, src_data, src_offset, count)
    }
}

impl Guarded for Memory {
    type Unguarded = UnguardedMem;
    type Guard = StoreGuard;

    unsafe fn from_unguarded(memory: UnguardedMem, guard: Self::Guard) -> Self {
        Self(Handle::from_unguarded(memory, guard))
    }

    fn to_unguarded(self, guard: Self::Guard) -> Self::Unguarded {
        self.0.to_unguarded(guard)
    }
}

/// An unguarded version of [`Mem`].
pub(crate) type UnguardedMem = UnguardedHandle<MemoryEntity>;

/// The type of a [`Memory`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct MemoryType {
    minimum: u32,
    maximum: Option<u32>,
}

impl MemoryType {
    /// Creates a new [`MemoryType`] with the following parameters:
    /// 
    /// * `minimum` - The [`Memory`]'s minimum size.
    /// * `maximum` - The [`Memory`]'s maximum size.
    pub fn new(minimum: u32, maximum: Option<u32>) -> Self {
        Self { minimum, maximum }
    }

    /// Returns the [`Memory`]'s minimum size.
    pub fn minimum(&self) -> u32 {
        self.minimum
    }

    /// Returns the [`Memory`]'s maximum size, if any.
    pub fn maximum(&self) -> Option<u32> {
        self.maximum
    }

    pub(crate) fn is_valid(&self) -> bool {
        let maximum = if let Some(maximum) = self.maximum {
            if maximum > 65_536 {
                return false;
            }
            maximum
        } else {
            65_536
        };
        if self.minimum > maximum {
            return false;
        }  
        true  
    }

    pub(crate) fn is_subtype_of(self, other: Self) -> bool {
        if self.minimum < other.minimum {
            return false;
        }
        match (self.maximum, other.maximum) {
            (None, Some(_)) => return false,
            (Some(maximum), Some(other_maximum)) if maximum > other_maximum => return false,
            _ => ()
        };
        true
    }
}

impl Decode for MemoryType {
    fn decode(decoder: &mut Decoder<'_>) -> Result<Self, DecodeError> {
        let has_maximum = match decoder.read_byte()? {
            0x00 => false,
            0x01 => true,
            _ => return Err(DecodeError::new("invalid memory type")),
        };
        Ok(Self {
            minimum: decoder.decode()?,
            maximum: if has_maximum {
                Some(decoder.decode()?)
            } else {
                None
            },
        })
    }
}

/// An error that can occur when operating on a [`Mem`].
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum MemoryError {
    FailedToGrow,
}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FailedToGrow => write!(f, "memory failed to grow"),
        }
    }
}

impl Error for MemoryError {}

/// The representation of a [`Mem`] in a [`Store`].
#[derive(Debug)]
pub(crate) struct MemoryEntity {
    maximum: Option<u32>,
    bytes: Vec<u8>,
}

impl MemoryEntity {
    fn new(minimum: u32, maximum: Option<u32>) -> Self {
        Self {
            maximum,
            bytes: vec![0; (minimum as usize).checked_mul(PAGE_SIZE).unwrap()],
        }
    }

    fn ty(&self) -> MemoryType {
        MemoryType {
            minimum: self.size(),
            maximum: self.maximum,
        }
    }

    pub(crate) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(crate) fn bytes_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }

    pub(crate) fn size(&self) -> u32 {
        u32::try_from(self.bytes.len() / PAGE_SIZE).unwrap()
    }

    pub(crate) fn grow(&mut self, stack: Option<&mut Stack>, count: u32) -> Result<u32, MemoryError> {
        unsafe { self.grow_with_stack(count, stack) }
    }

    pub(crate) unsafe fn grow_with_stack(
        &mut self,
        count: u32,
        stack: Option<&mut Stack>,
    ) -> Result<u32, MemoryError> {
        if count > self.maximum.unwrap_or(65_536) - self.size() {
            return Err(MemoryError::FailedToGrow);
        }
        let old_data = self.bytes.as_mut_ptr();
        let old_size = self.size();
        let new_size = self.size() + count;
        self.bytes
            .resize((new_size as usize).checked_mul(PAGE_SIZE).unwrap(), 0);
        let new_data = self.bytes.as_mut_ptr();

        if let Some(stack) = stack {
            // Each call frame on the stack stores the value of the `md` and `ms` register. Growing
            // this [`Memory`] invalidates all call frames for which `md` and `ms` store a pointer to
            // the old data and size of this [`Memory`]. To fix this, we need to iterate over the call
            // frames on the stack, and update the value of the `md` and `ms` register to store
            // a pointer to the new data and size of this [`Memory`] instead.
            let mut ptr = stack.as_mut_ptr().add(stack.len());
            while ptr != stack.as_mut_ptr() {
                let saved_regs: &mut SavedRegs = &mut *ptr.offset(-(size_of::<SavedRegs>() as isize)).cast();
                ptr = saved_regs.sp;
                if saved_regs.md == old_data {
                    saved_regs.md = new_data;
                    saved_regs.ms = new_size;
                }
            }
        }

        Ok(old_size)
    }

    pub(crate) fn fill(&mut self, idx: u32, val: u8, num: u32) -> Result<(), Trap> {
        let bytes = self
            .bytes
            .get_mut(idx as usize..)
            .and_then(|bytes| bytes.get_mut(..num as usize))
            .ok_or(Trap::MemAccessOutOfBounds)?;
        bytes.fill(val);
        Ok(())
    }

    pub(crate) fn copy_within(
        &mut self,
        dst_idx: u32,
        src_idx: u32,
        num: u32,
    ) -> Result<(), Trap> {
        let size = self.bytes.len() as u32;
        if num > size || dst_idx > size - num || src_idx > size - num {
            return Err(Trap::MemAccessOutOfBounds);
        }
        self.bytes.copy_within(
            src_idx as usize..src_idx as usize + num as usize,
            dst_idx as usize
        );
        Ok(())
    }

    pub(crate) fn init(
        &mut self,
        dst_idx: u32,
        src_data: &DataEntity,
        src_idx: u32,
        num: u32,
    ) -> Result<(), Trap> {
        let dst_bytes = self
            .bytes
            .get_mut(dst_idx as usize..)
            .and_then(|bytes| bytes.get_mut(..num as usize))
            .ok_or(Trap::MemAccessOutOfBounds)?;
        let src_bytes = src_data
            .bytes()
            .get(src_idx as usize..)
            .and_then(|bytes| bytes.get(..num as usize))
            .ok_or(Trap::MemAccessOutOfBounds)?;
        dst_bytes.copy_from_slice(src_bytes);
        Ok(())
    }
}

const PAGE_SIZE: usize = 65_536;
