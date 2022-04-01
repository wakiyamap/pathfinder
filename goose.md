# Slow RPC-API debugging report

A user reported running a load test and getting and capping out at ~25 requests/s. Which is pretty abysmal. We haven't actually done any load testing at all, so maybe this shouldn't be unexpected. The user used [locust](https://locust.io/) which is a very easy to use python framework for load testing.

The user ran a combination of 5 different API calls, all equally split. He since deleted that message so I don't know exactly which ones, and I don't know what parameters he used (valid or invalid hashes?).

User was also running a fully syncd node on mainnet, so db write access "should" be quite limited.

## Methodology

I've found a rust `locust` clone built in rust, [goose](https://docs.rs/goose/latest/goose/). I'll be using this to replicate the users report and investigate causes (if any).

I'll be adding a commit for each test run which will include source code changes and results.

## Pre-testing thoughts

Possible culprits:
- python :D `locust` might be slow -- which it probably is, but not 25rps slow..
- sqlite WAL -- we haven't enabled this.. known multi-access performance boost.
- lack of indexing in some tables.. although could it really be this bad?
-