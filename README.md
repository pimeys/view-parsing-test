# View SQL parsing tests

A repo to try out can we diff SQL from views. It creates a view to the database,
reads it back and expects it to match with the input statement.

## How to run

Start the database:

```bash
> docker compose up -d
```

or

```bash
> podman-compose up -d
```

Then execute the tests:

```bash
> cargo test
```

If wanting to update the expected queries set `UPDATE_EXPECT=1` before running
the tests.

## How to add new tests

The directory `schemas` has subdirectories, one per test. First the test creates a
schema with the directory name, after which it executes the `setup.sql`. Finally
the test asserts the `expected.sql` to match with the information schema. To add new
tests, create a new subdirectory and add the `setup.sql` and `expected.sql`
files.