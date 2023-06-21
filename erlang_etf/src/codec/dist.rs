bitflags::bitflags! {
    /// See [erts/emulator/beam/dist.h](https://github.com/erlang/otp/blob/OTP-25.0.3/erts/emulator/beam/dist.h) in the Erlang/OTP source code.
    pub struct DistributionFlags: u64 {
        const DFLAG_PUBLISHED = 0x01;
        const DFLAG_ATOM_CACHE = 0x02;
        const DFLAG_EXTENDED_REFERENCES = 0x04;
        const DFLAG_DIST_MONITOR = 0x08;
        const DFLAG_FUN_TAGS = 0x10;
        const DFLAG_DIST_MONITOR_NAME = 0x20;
        const DFLAG_HIDDEN_ATOM_CACHE = 0x40;
        const DFLAG_NEW_FUN_TAGS = 0x80;
        const DFLAG_EXTENDED_PIDS_PORTS = 0x100;
        const DFLAG_EXPORT_PTR_TAG = 0x200;
        const DFLAG_BIT_BINARIES = 0x400;
        const DFLAG_NEW_FLOATS = 0x800;
        const DFLAG_UNICODE_IO = 0x1000;
        const DFLAG_DIST_HDR_ATOM_CACHE = 0x2000;
        const DFLAG_SMALL_ATOM_TAGS = 0x4000;
        const DFLAG_ETS_COMPRESSED = 0x8000; /* internal */
        const DFLAG_UTF8_ATOMS = 0x10000;
        const DFLAG_MAP_TAG = 0x20000;
        const DFLAG_BIG_CREATION = 0x40000;
        const DFLAG_SEND_SENDER = 0x80000;
        const DFLAG_BIG_SEQTRACE_LABELS = 0x100000;
        const DFLAG_PENDING_CONNECT = 0x200000; /* internal */
        const DFLAG_EXIT_PAYLOAD = 0x400000;
        const DFLAG_FRAGMENTS = 0x800000;
        const DFLAG_HANDSHAKE_23 = 0x1000000;
        const DFLAG_UNLINK_ID = 0x2000000;
        const DFLAG_MANDATORY_25_DIGEST = 0x4000000;
        const DFLAG_RESERVED = 0xf8000000;
        /*
         * As the old handshake only support 32 flag bits, we reserve the remaining
         * bits in the lower 32 for changes in the handshake protocol or potentially
         * new capabilities that we also want to backport to OTP-22 or older.
         */
        const DFLAG_SPAWN = 0x1 << 32;
        const DFLAG_NAME_ME = 0x2 << 32;
        const DFLAG_V4_NC = 0x4 << 32;
        const DFLAG_ALIAS = 0x8 << 32;
        /*
         * In term_to_binary/2, we will use DFLAG_ATOM_CACHE to mean
         * DFLAG_DETERMINISTIC.
         */
        const DFLAG_DETERMINISTIC = Self::DFLAG_ATOM_CACHE.bits;
        /* Mandatory flags for distribution in OTP 25. */
        const DFLAG_DIST_MANDATORY_25 =
            ( Self::DFLAG_EXTENDED_REFERENCES.bits
            | Self::DFLAG_FUN_TAGS.bits
            | Self::DFLAG_EXTENDED_PIDS_PORTS.bits
            | Self::DFLAG_UTF8_ATOMS.bits
            | Self::DFLAG_NEW_FUN_TAGS.bits
            | Self::DFLAG_BIG_CREATION.bits
            | Self::DFLAG_NEW_FLOATS.bits
            | Self::DFLAG_MAP_TAG.bits
            | Self::DFLAG_EXPORT_PTR_TAG.bits
            | Self::DFLAG_BIT_BINARIES.bits
            | Self::DFLAG_BIT_BINARIES.bits
            | Self::DFLAG_HANDSHAKE_23.bits);
        /* New mandatory flags for distribution in OTP 26 */
        const DFLAG_DIST_MANDATORY_26 =
            ( Self::DFLAG_V4_NC.bits
            | Self::DFLAG_UNLINK_ID.bits);
        /* Mandatory flags for distribution. */
        const DFLAG_DIST_MANDATORY =
            ( Self::DFLAG_DIST_MANDATORY_25.bits
            | Self::DFLAG_DIST_MANDATORY_26.bits);
        /*
         * Additional optimistic flags when encoding toward pending connection.
         * If remote node (erl_interface) does not support these then we may need
         * to transcode messages enqueued before connection setup was finished.
         */
        const DFLAG_DIST_HOPEFULLY =
            ( Self::DFLAG_DIST_MONITOR.bits
            | Self::DFLAG_DIST_MONITOR_NAME.bits
            | Self::DFLAG_SPAWN.bits
            | Self::DFLAG_ALIAS.bits);
        /* Our preferred set of flags. Used for connection setup handshake */
        const DFLAG_DIST_DEFAULT =
            ( Self::DFLAG_DIST_MANDATORY.bits
            | Self::DFLAG_DIST_HOPEFULLY.bits
            | Self::DFLAG_UNICODE_IO.bits
            | Self::DFLAG_DIST_HDR_ATOM_CACHE.bits
            | Self::DFLAG_SMALL_ATOM_TAGS.bits
            | Self::DFLAG_SEND_SENDER.bits
            | Self::DFLAG_BIG_SEQTRACE_LABELS.bits
            | Self::DFLAG_EXIT_PAYLOAD.bits
            | Self::DFLAG_FRAGMENTS.bits
            | Self::DFLAG_SPAWN.bits
            | Self::DFLAG_ALIAS.bits
            | Self::DFLAG_MANDATORY_25_DIGEST.bits);
        /* Flags addable by local distr implementations */
        const DFLAG_DIST_ADDABLE = Self::DFLAG_DIST_DEFAULT.bits;
        /* Flags rejectable by local distr implementation */
        const DFLAG_DIST_REJECTABLE =
            ( Self::DFLAG_DIST_HDR_ATOM_CACHE.bits
            | Self::DFLAG_HIDDEN_ATOM_CACHE.bits
            | Self::DFLAG_FRAGMENTS.bits
            | Self::DFLAG_ATOM_CACHE.bits);
        /* Flags for all features needing strict order delivery */
        const DFLAG_DIST_STRICT_ORDER = Self::DFLAG_DIST_HDR_ATOM_CACHE.bits;
        /* All flags that should be enabled when term_to_binary/1 is used. */
        const TERM_TO_BINARY_DFLAGS = Self::DFLAG_NEW_FLOATS.bits;
    }
}
