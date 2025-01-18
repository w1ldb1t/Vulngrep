
<h1 align="center">
  <br>
  Github Notify
  <br>
</h1>

<h4 align="center">A tool for monitoring GitHub repositories and alerting users based on customizable patterns.</h4>

<p align="center">
  <a href="https://github.com/w1ldb1t/github-notify/actions/workflows/release.yml">
    <img src="https://github.com/w1ldb1t/github-notify/actions/workflows/release.yml/badge.svg">
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
$ git clone https://github.com/w1ldb1t/github-notify.git

# Go into the repository
$ cd github-notify

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
      - path: drivers/phy/phy-core.c
      - path: fs/btrfs/sysfs.c
```

## Download

You can [download](https://github.com/w1ldb1t/github-notify/releases) the latest installable version of `github-notify` for Windows and Linux.

## License

This project is licensed under "GPL-3.0 license " License - see the [LICENSE](LICENSE) file for details.