# GitHub RSS Aggregator

Welcome to the **GitHub RSS Aggregator** – a simple, open-source, GitHub-hosted RSS feed aggregator written in Rust. It fetches multiple RSS feeds concurrently, aggregates and sorts their items, and generates an OPML feed list file (`feeds/master.opml`). It requires no backend or database—just pure goodness running completely in your GitHub repo!

## Features

- **Multi-Feed Support:** Reads feed URLs from a `feeds.txt` file (one URL per line).
- **Configurable:** Uses a `config.toml` file to set options (currently, the maximum number of items to sync - set to 0 for unlimited).
- **Concurrent Fetching:** Uses asynchronous Rust with Tokio to fetch feeds in parallel.
- **OPML Feed List Creation: Creates an OPML file listing all feeds with their metadata instead of aggregating content.
- **GitHub Actions Integration:**
  - **Build Release:** Automatically builds the release binary on pull requests.
  - **Update Master Feed:** Periodically runs the aggregator, updating `feeds/master.opml` and pushing changes back to the repo.
  - **Direct RSS Link:** The GitHub link to the `feeds/master.opml` file can be imported into RSS readers that support OPML subscription lists.

## Usage

- Update link to repo in feeds/master.opml and in GitHub Action.
- Add URLs (one per line) to the `feeds.txt` file and commit the changes.

## Configuration

The `config.toml` file allows you to configure the RSS aggregator:

- `max_items`: Maximum number of items to include in feeds. Set to `0` for unlimited items (default: 300 if not specified). This limit applies to both the master feed and individual feed files.
- `repo_name`: GitHub repository name in format `owner/repo` (optional, default: "xavwe/rss-aggregator"). Used for generating feed URLs in RSS channels.

## Setting Up the Project with a PAT

For the GitHub Actions workflows to successfully create releases and push updates (such as updating `feeds/master.opml`), you need to configure a Personal Access Token (PAT) and add it as a secret named `RELEASE_TOKEN` in your repository. This token is used by the workflows to authenticate operations that modify the repository.

### Required PAT Permissions

Your PAT must have the following scopes:
- **repo** — Grants full control of private repositories (and is sufficient for public repositories as well). This scope allows the workflows to:
  - Create and upload release assets.
  - Push changes (for example, updating `feeds/master.opml`).

### Configuring the PAT

1. **Create a PAT:**
  - Go to your GitHub account's **Settings > Developer settings > Personal access tokens**.
  - Click on **Generate new token**.
  - Select the **repo** scope (and any additional scopes if needed) and generate the token.
  - **Copy the token** (you won’t be able to see it again).

2. **Add the PAT as a Repository Secret:**
  - In your repository, navigate to **Settings > Secrets and variables > Actions > New repository secret**.
  - Name the secret `RELEASE_TOKEN` and paste your PAT as its value.
  - Save the secret.

Once configured, the workflows will automatically use the `RELEASE_TOKEN` secret for release creation and pushing updates to the repository.

Enjoy aggregating your RSS feeds and happy coding! 🚀
