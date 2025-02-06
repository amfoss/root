<div align="center">
  <h1>Root</h1>
  <p>A GraphQL backend for managing club member information</p>
</div>

---

Root is our club's backend, responsible for collecting and distributing data from and to all the other services including [Home](https://www.github.com/amfoss/home), [amD](https://www.github.com/amfoss/amd) and [Presense](https://www.github.com/amfoss/presense). The idea is to have all our data easily available in one place and to let every other end-user applications to be standalone. This ensures there's no single point of failure for all our infrastructure (as was the case with our previous CMS). Though Root going down would definitely cause a few features to stop working on the other apps.

# Quick Setup

1. Install prerequisites:
   - Rust (latest stable should work fine)
   - PostgreSQL
   - SQLx CLI: `cargo install sqlx-cli`

2. Configure environment:
   ```bash
   cp .env.sample .env
   ```
   - Make sure that you have a postgres database running with the specified credentials.

3. Setup database:
   ```bash
   sqlx database create
   sqlx migrate run
   ```

4. Run server:
   ```bash
   cargo run
   ```

GraphQL playground should be available at `http://localhost:8000/graphiql` as long as it's in development mode.


# Deployment
The deployed instance can be accessed at [root.amfoss.in](https://root.amfoss.in).

The `main` branch is exclusively meant for production use and commits which get merged into it will make their way into the deployed instance. Active development should occur on the `develop` branch and when sufficient stability has been achieved, they can be merged into `main`. This will kick off the deployment workflow. 

Further implementation details can be found at [bedrock](https://github.com/amfoss/bedrock).

# Documentation

See the [documentation](docs/docs.md) for the API reference, database schema and other detailed documentation.  

# Contributing

## Reporting Issues

If you encounter a bug, please check existing issues first to avoid duplicates. If none exist, create a new issue with the following details:

* Title: Concise summary.
* Description: A detailed description of the issue.
* Steps to Reproduce: If it's a bug, include steps to reproduce.
* Expected and Actual Behavior: Describe what you expected and what actually happened.

## Suggesting Features

We welcome new ideas! Please open an issue titled "Feature Request: `<Feature Name>`" and provide:

* Problem: What problem does this feature solve?
* Solution: Describe how you envision it working.
* Alternatives Considered: Mention any alternatives you've considered.

## Submitting Code Changes

If you'd like to fix a bug, add a feature, or improve code quality:

* Check the open issues to avoid redundancy.
* Open a draft PR if you'd like feedback on an ongoing contribution.
* **Make sure to set the `develop` branch as your pull request target**, see [Deployment](#deployment)

## Coding Standards

* Follow Rust Conventions: Use idiomatic Rust patterns. Use `cargo fmt` and `cargo clippy` to format and lint your code.
* Modularity: Write modular, reusable functions. Avoid monolithic code.
* Descriptive Naming: Use descriptive names for variables, functions, and types.
* Don't worry too much about rules, it just needs to be pretty. Most editors have built-in tools to do this for you. 

# License

This project is licensed under GNU General Public License V3. You are welcome to adapt it, make it yours. Just make sure that you credit us too.
