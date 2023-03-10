use core::{marker::PhantomData, mem::size_of};

use embedded_hal::blocking::rng::Read;

use rand::{RngCore, SeedableRng};

use crate::hwrng::HWRNG;

/// Wrapper around RIOTs `random` module.
///
/// ## Seedlength
/// Since the module allows a dynamic seedsize
/// it needs to be specified in the type.
/// The `SEED_LENGTH` variable specifies the seedlength in bytes.
/// **Since RIOT takes in `uint32_t` (`u32`) the length need to be divisable by 4!**
///
/// ## Security
/// Even though `Random` claims to be a cryptographic secure prng
/// it only can be if provided sufficently random seeds! See remarks at [`crate::hwrng::HWRNG`]
/// if when using it to generate seeds.
///
/// ## Global state
/// Be aware that there should be only one `Random` object at a time,
/// since RIOT uses a global state for this internally, so creating a second object
/// just results in the global state beeing overwritten and
/// both objects representing practically the same prng.
#[derive(Debug)]
pub struct Random<const SEED_LENGTH: usize> {
    // Make sure this gets not manually constructed
    private: PhantomData<()>,
}

impl<const SEED_LENGTH: usize> RngCore for Random<SEED_LENGTH> {
    fn next_u32(&mut self) -> u32 {
        unsafe { riot_sys::random_uint32() }
    }

    fn next_u64(&mut self) -> u64 {
        rand_core::impls::next_u64_via_u32(self)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        unsafe { riot_sys::random_bytes(dest.as_mut_ptr() as *mut _, dest.len() as u32) }
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.fill_bytes(dest);
        Ok(())
    }
}

/// A seed of length `SEED_LENGTH` inteded to be used by [`Random`].
///
/// ## Seedlength
/// Since [`Random`] allows a dynamic seedsize
/// it needs to be specified in the type.
/// The `SEED_LENGTH` variable specifies the seedlength in bytes.
/// **Since RIOT takes in `uint32_t` (`u32`) the length need to be divisable by 4!**
///
/// ## Security
/// This is only a container for a seed and therefore
/// can not give any assurances as to the quality of the contained seed,
/// which wholly depends on the method with which the contained seed was generated.  
#[derive(Debug)]
pub struct RandomSeed<const SEED_LENGTH: usize> {
    seed: [u8; SEED_LENGTH],
}

impl<const SEED_LENGTH: usize> RandomSeed<SEED_LENGTH> {
    // Workaround: see https://github.com/nvzqz/static-assertions-rs/issues/40#issuecomment-1458897730
    const CHECK_DIVISIBLE_BY_FOUR: () = assert!(SEED_LENGTH & 3 == 0);

    /// Creates an empty (zeroed) seedcontainer.
    ///
    /// This should **not** be used as a seed for anything that
    /// should provide any security. It is only meant to setup the buffer,
    /// which then can be accessed via its `buffer()` method.
    pub fn new_empty() -> Self {
        // Needed here to force the evaluation of the const
        let _ = Self::CHECK_DIVISIBLE_BY_FOUR;

        RandomSeed {
            seed: [0; SEED_LENGTH],
        }
    }

    /// Creates a [`RandomSeed`] with a seed generated by
    /// [`crate::hwrng::HWRNG`].
    ///
    /// See remakrs there on the quality of the
    /// generated seeds which depends very much on the used board.
    pub fn new_from_hwrng() -> Self {
        let mut seed = RandomSeed::<SEED_LENGTH>::default();

        unsafe {
            HWRNG.read(&mut seed.buffer()).unwrap_unchecked();
        }

        seed
    }

    /// The internal buffer
    pub fn buffer(&mut self) -> &mut [u8] {
        &mut self.seed
    }
}

// Enforced by `rand::SeedableRng`
impl<const SEED_LENGTH: usize> Default for RandomSeed<SEED_LENGTH> {
    fn default() -> Self {
        Self::new_empty()
    }
}

// Enforced by `rand::SeedableRng`
impl<const SEED_LENGTH: usize> AsMut<[u8]> for RandomSeed<SEED_LENGTH> {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.seed
    }
}

impl<const SEED_LENGTH: usize> SeedableRng for Random<SEED_LENGTH> {
    type Seed = RandomSeed<SEED_LENGTH>;

    fn from_seed(mut seed: Self::Seed) -> Self {
        unsafe {
            riot_sys::random_init_by_array(
                seed.seed.as_mut_ptr() as *mut u32,
                (seed.seed.len() / size_of::<i32>()) as i32,
            );
        }
        Random {
            private: PhantomData,
        }
    }
}
