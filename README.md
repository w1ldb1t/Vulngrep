
<h1 align="center">
  <br>
  Vulngrep
  <br>
</h1>

<h4 align="center">A tool for monitoring GitHub repositories and alerting users based on customizable patterns.</h4>

<p align="center">
  <a href="https://github.com/w1ldb1t/vulngrep/actions/workflows/release.yml">
    <img src="https://github.com/w1ldb1t/vulngrep/actions/workflows/release.yml/badge.svg">
  </a>
</p>

<p align="center">
  <a href="#description">Description</a> •
  <a href="#privacy">Privacy</a> •
  <a href="#how-to-use">How To Use</a> •
  <a href="#example-config">Example Config</a> •
  <a href="#download">Download</a> •
  <a href="#license">License</a>
</p>

## Description
The main goal of this tool is to help researchers stay up-to-date with the latest commits on open-source projects hosted by GitHub. The tool works exclusively using network requests through [Octocrab](https://github.com/XAMPPRocky/octocrab), a third-party GitHub API client.

## Privacy

The tool downloads all commit details to the user's local machine for processing. This approach distributes API traffic evenly across all commits, avoiding targeted queries on the history of specific files. By doing so, it prevents revealing heightened user interest in particular files through API activity.

## How To Use

To clone and run this application, you'll need [Git](https://git-scm.com/) and [Rust](https://www.rust-lang.org/) (which includes Cargo, Rust's package manager) installed on your computer. From your command line:

```bash
# Clone this repository
$ git clone https://github.com/w1ldb1t/vulngrep.git

# Go into the repository
$ cd vulngrep

# Build the app
$ cargo build

# Run the app
$ cargo run

# Configure the app
$ cargo run config
```

## Example config

```yaml
interval: 6h # optional
github_token: GITHUB_TOKEN
notifications:
  - repository:
      name: linux
      owner: torvalds
    files:
      - path: fs/btrfs/sysfs.c
      - path: drivers/phy/phy-core.c
        pattern:
          # per-file patterns
          - "refcount_add(*)"
          - "refcount_add_not_zero(*)"
    pattern:
      # global repository patterns
      - "UAF"
      - "Overflow"
```

## Download

You can [download](https://github.com/w1ldb1t/vulngrep/releases) the latest installable version of `vulngrep` for Windows and Linux.

## License

This project is licensed under "GPL-3.0 license " License - see the [LICENSE](LICENSE) file for details.