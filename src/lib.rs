pub trait Hasher {
    type Checksum;
    type State;

    fn width(&self) -> usize;
    fn empty_checksum() -> Self::Checksum;
    fn initial_state() -> Self::State;
    fn process_byte(
        &self,
        state: Self::State,
        old_byte: u8,
        new_byte: u8,
    ) -> (Self::Checksum, Self::State);

    fn process_slice(
        &self,
        state: Self::State,
        old_data: &[u8],
        new_data: &[u8],
    ) -> (Self::Checksum, Self::State) {
        old_data.iter().zip(new_data.iter()).fold(
            (Self::empty_checksum(), state),
            |(_, prev_state), (old_byte, new_byte)| {
                self.process_byte(prev_state, *old_byte, *new_byte)
            },
        )
    }

    fn process_chunk64(
        &self,
        state: Self::State,
        old_data: &[u8; 8],
        new_data: &[u8; 8],
    ) -> (Self::Checksum, Self::State) {
        self.process_slice(state, old_data, new_data)
    }

    fn process_chunk128(
        &self,
        state: Self::State,
        old_data: &[u8; 16],
        new_data: &[u8; 16],
    ) -> (Self::Checksum, Self::State) {
        self.process_slice(state, old_data, new_data)
    }

    fn process_chunk256(
        &self,
        state: Self::State,
        old_data: &[u8; 32],
        new_data: &[u8; 32],
    ) -> (Self::Checksum, Self::State) {
        self.process_slice(state, old_data, new_data)
    }

    fn process_chunk512(
        &self,
        state: Self::State,
        old_data: &[u8; 64],
        new_data: &[u8; 64],
    ) -> (Self::Checksum, Self::State) {
        self.process_slice(state, old_data, new_data)
    }
}

pub struct Rolling<H, I>
where
    H: Hasher,
{
    hasher: H,
    next: Option<H::Checksum>,
    state: H::State,
    begin: usize,
    ring: Vec<u8>,
    bytes: I,
}

impl<H, I> Rolling<H, I>
where
    H: Hasher,
    I: Iterator<Item = u8>,
{
    pub fn start(hasher: H, mut it: I) -> Option<Self> {
        let mut hold = (H::empty_checksum(), H::initial_state());
        let mut i = 0;
        let mut ring = Vec::with_capacity(hasher.width());

        while let Some(byte) = it.next() {
            if i == hasher.width() {
                break;
            }

            let (_, state) = hold;
            hold = hasher.process_byte(state, 0, byte);
            ring.push(byte);
            i += 1;
        }
        let (sum, state) = hold;

        if i == hasher.width() {
            Some(Self {
                hasher,
                next: Some(sum),
                state,
                begin: 0,
                ring,
                bytes: it,
            })
        } else {
            None
        }
    }

    fn feed(&mut self, byte: u8) -> H::Checksum {
        // mild hack
        let mut dummy = H::initial_state();
        std::mem::swap(&mut dummy, &mut self.state);
        let prev_state = dummy;

        let (sum, new_state) = self
            .hasher
            .process_byte(prev_state, self.ring[self.begin], byte);
        self.state = new_state;
        self.ring[self.begin] = byte;
        self.begin = (self.begin + 1) % self.hasher.width();

        sum
    }
}

impl<H, I> Iterator for Rolling<H, I>
where
    H: Hasher,
    I: Iterator<Item = u8>,
{
    type Item = H::Checksum;

    fn next(&mut self) -> Option<Self::Item> {
        let mut new_next = if let Some(byte) = self.bytes.next() {
            Some(self.feed(byte))
        } else {
            None
        };

        std::mem::swap(&mut self.next, &mut new_next);
        let old_next = new_next;

        old_next
    }
}

pub mod rrs;
