-- The `souvenir` type, its operators, operator classes and casts.
--
-- Runs after every function in src/lib.rs exists (`finalize`). The shell
-- type was created first (`bootstrap`) so those functions could name it.

CREATE TYPE souvenir (
    INPUT = souvenir_in,
    OUTPUT = souvenir_out,
    RECEIVE = souvenir_recv,
    SEND = souvenir_send,
    INTERNALLENGTH = 20,
    ALIGNMENT = char,
    STORAGE = plain
);

COMMENT ON TYPE souvenir IS
    'Souvenir ID: prefix, timestamp and random bits in 20 bytes; prints as prefix_suffix';

-- Comparison operators. IDs sort by prefix, then creation time (byte order).

CREATE OPERATOR = (
    LEFTARG = souvenir, RIGHTARG = souvenir, FUNCTION = souvenir_eq,
    COMMUTATOR = =, NEGATOR = <>,
    RESTRICT = eqsel, JOIN = eqjoinsel, HASHES, MERGES
);

CREATE OPERATOR <> (
    LEFTARG = souvenir, RIGHTARG = souvenir, FUNCTION = souvenir_ne,
    COMMUTATOR = <>, NEGATOR = =,
    RESTRICT = neqsel, JOIN = neqjoinsel
);

CREATE OPERATOR < (
    LEFTARG = souvenir, RIGHTARG = souvenir, FUNCTION = souvenir_lt,
    COMMUTATOR = >, NEGATOR = >=,
    RESTRICT = scalarltsel, JOIN = scalarltjoinsel
);

CREATE OPERATOR <= (
    LEFTARG = souvenir, RIGHTARG = souvenir, FUNCTION = souvenir_le,
    COMMUTATOR = >=, NEGATOR = >,
    RESTRICT = scalarlesel, JOIN = scalarlejoinsel
);

CREATE OPERATOR > (
    LEFTARG = souvenir, RIGHTARG = souvenir, FUNCTION = souvenir_gt,
    COMMUTATOR = <, NEGATOR = <=,
    RESTRICT = scalargtsel, JOIN = scalargtjoinsel
);

CREATE OPERATOR >= (
    LEFTARG = souvenir, RIGHTARG = souvenir, FUNCTION = souvenir_ge,
    COMMUTATOR = <=, NEGATOR = <,
    RESTRICT = scalargesel, JOIN = scalargejoinsel
);

-- Default operator classes: B-tree (primary keys, ORDER BY, range scans)
-- and hash (hash joins, hash indexes, DISTINCT).

CREATE OPERATOR CLASS souvenir_ops
    DEFAULT FOR TYPE souvenir USING btree AS
        OPERATOR 1 <,
        OPERATOR 2 <=,
        OPERATOR 3 =,
        OPERATOR 4 >=,
        OPERATOR 5 >,
        FUNCTION 1 souvenir_cmp(souvenir, souvenir);

CREATE OPERATOR CLASS souvenir_hash_ops
    DEFAULT FOR TYPE souvenir USING hash AS
        OPERATOR 1 =,
        FUNCTION 1 souvenir_hash(souvenir),
        FUNCTION 2 souvenir_hash_extended(souvenir, bigint);

-- Casts. Text converts both ways on assignment, so `INSERT ... VALUES ($1)`
-- with a text parameter works; elsewhere write `$1::souvenir`.

CREATE CAST (text AS souvenir) WITH INOUT AS ASSIGNMENT;
CREATE CAST (varchar AS souvenir) WITH INOUT AS ASSIGNMENT;
CREATE CAST (souvenir AS text) WITH INOUT AS ASSIGNMENT;
CREATE CAST (souvenir AS bytea) WITH FUNCTION souvenir_to_bytea(souvenir);
CREATE CAST (bytea AS souvenir) WITH FUNCTION souvenir_from_bytea(bytea);
