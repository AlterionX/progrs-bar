use crossterm::style::{SetForegroundColor, Color};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum BucketSize {
    Large,
    Small,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BucketDescription {
    size: usize,
    len: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BucketInfo {
    larger: BucketDescription,
    smaller: BucketDescription,

    filled_portion: BucketSize,
    #[allow(unused)] // Included for completion.
    covered_by_larger: usize,
    partial_contents: usize,
}

impl BucketInfo {
    pub fn create(total: usize, filled: usize, bucket_len: usize) -> Self {
        let smaller_bucket_size = total / bucket_len;
        let larger_bucket_size = smaller_bucket_size + 1;

        // The remainder is distributed one by one to each bucket.
        // This makes the largest size for a larger bucket 1 larger than the smaller bucket.
        let larger_bucket_len = total % bucket_len;
        let smaller_bucket_len = bucket_len - larger_bucket_len;

        let covered_by_larger = larger_bucket_len * larger_bucket_size;
        let (partial_contents, filled_portion) = if covered_by_larger >= filled {
            (filled, BucketSize::Large)
        } else {
            (filled - covered_by_larger, BucketSize::Small)
        };

        Self {
            larger: BucketDescription {
                size: larger_bucket_size,
                len: larger_bucket_len,
            },
            smaller: BucketDescription {
                size: smaller_bucket_size,
                len: smaller_bucket_len,
            },

            covered_by_larger,
            filled_portion,
            partial_contents,
        }
    }
}

pub struct Bar {
    max: usize,
    val: usize,
}

impl Bar {
    pub fn new(val: usize, max: usize) -> Self {
        debug_assert!(val <= max);
        Self {
            max,
            val,
        }
    }
}

impl Bar {
    const EMPTY_BUCKET_CHAR: char = ' ';
    const FILLED_BUCKET_CHAR: char = '█';
    const PARTIAL_BUCKETS_CHARS: [char; 8] = ['▏', '▎', '▍', '▌', '▋', '▊', '▉', Self::FILLED_BUCKET_CHAR];

    pub fn generate_string(&self, available_space: usize, color: Color) -> String {
        // We need at least 2 spaces for margins.
        if available_space <= 2 {
            return String::new();
        }

        let bar_len = available_space - 2;
        let buckets = if self.max < bar_len {
            // Hack to cover when each individual character represents more than one tick.
            //
            // Assumes self.max < usize::MAX / 500
            //
            // TODO Fix for real.
            let base_virtual_multiplier = bar_len / self.max;
            let virtual_multiplier = if self.max.is_multiple_of(bar_len) {
                base_virtual_multiplier + 1
            } else {
                base_virtual_multiplier
            };
            BucketInfo::create(self.max * virtual_multiplier, self.val * virtual_multiplier, bar_len)
        } else {
            BucketInfo::create(self.max, self.val, bar_len)
        };

        match buckets.filled_portion {
            BucketSize::Large => {
                std::iter::once('[')
                    .chain(SetForegroundColor(color).to_string().chars())
                    .chain(Self::partial_health_bar(buckets.partial_contents, buckets.larger.len, buckets.larger.size))
                    .chain(std::iter::repeat_n(Self::EMPTY_BUCKET_CHAR, buckets.smaller.len))
                    .chain(SetForegroundColor(Color::Reset).to_string().chars())
                    .chain(std::iter::once(']'))
                    .collect()
            },
            BucketSize::Small => {
                std::iter::once('[')
                    .chain(SetForegroundColor(color).to_string().chars())
                    .chain(std::iter::repeat_n(Self::FILLED_BUCKET_CHAR, buckets.larger.len))
                    .chain(Self::partial_health_bar(buckets.partial_contents, buckets.smaller.len, buckets.smaller.size))
                    .chain(SetForegroundColor(Color::Reset).to_string().chars())
                    .chain(std::iter::once(']'))
                    .collect()
            },
        }
    }

    fn partial_health_bar(hp: usize, bar_len: usize, bucket_size: usize) -> impl Iterator<Item=char> {
        let partial_bucket_hp = hp % bucket_size;
        let filled_buckets = hp / bucket_size;

        let empty_buckets = if partial_bucket_hp == 0 {
            bar_len - filled_buckets
        } else {
            bar_len - filled_buckets - 1
        };

        let partial_hp_bucket_char = {
            let subbuckets = BucketInfo::create(bucket_size, partial_bucket_hp, Self::PARTIAL_BUCKETS_CHARS.len());

            let idx = match subbuckets.filled_portion {
                BucketSize::Large => {
                    subbuckets.partial_contents / subbuckets.larger.size
                },
                BucketSize::Small => {
                    subbuckets.larger.len + (subbuckets.partial_contents / subbuckets.smaller.size)
                },
            };

            Self::PARTIAL_BUCKETS_CHARS[idx]
        };

        let filled = std::iter::repeat_n(Self::FILLED_BUCKET_CHAR, filled_buckets);
        let empty = std::iter::repeat_n(Self::EMPTY_BUCKET_CHAR, empty_buckets);
        let partial = std::iter::repeat_n(partial_hp_bucket_char, if partial_bucket_hp == 0 { 0 } else { 1 });

        filled.chain(partial).chain(empty)
    }
}
