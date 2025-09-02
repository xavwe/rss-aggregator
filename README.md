# GitHub RSS Feed Archiver

Welcome to the **GitHub RSS Feed Archiver** – a simple, open-source, GitHub-hosted RSS feed archiver written in Rust. It fetches multiple RSS feeds, archives their complete history, and generates both individual archived feeds and an OPML subscription file that points to your archived feeds instead of the original limited ones. It requires no backend or database—just pure goodness running completely in your GitHub repo!

## Purpose

Many RSS feeds only contain the latest 10-20 articles, meaning you lose access to older content over time. This tool solves that problem by:

- **Archiving Complete Feed History:** Downloads and preserves all articles from RSS feeds, not just the latest ones
- **Individual Feed Archives:** Creates separate archived XML files for each feed with full article history
- **OPML Generation:** Creates an OPML subscription file that points RSS readers to your archived feeds instead of the originals

## Features

- **Multi-Feed Support:** Reads feed URLs from a `feeds.txt` file (one URL per line).
- **Complete Feed Archiving:** Preserves full article history, not just recent items like original feeds.
- **Configurable:** Uses a `config.toml` file to set options (maximum number of items to archive - set to 0 for unlimited).
- **Concurrent Fetching:** Uses asynchronous Rust with Tokio to fetch feeds in parallel.
- **Individual Feed Files:** Creates separate archived XML files for each feed source.
- **OPML Subscription File:** Generates `feeds/master.opml` that points RSS readers to your archived feeds.
- **GitHub Actions Integration:**
  - **Build Release:** Automatically builds and creates releases with the archiver binary on pushes and pull requests.
  - **Update Archives:** Downloads the latest release binary and runs it hourly to update all feed archives and the OPML file.
  - **Direct Import:** Import the generated OPML file directly into any RSS reader to access your complete archived feeds.

## Usage

1. **Fork this repository** to your own GitHub account.
2. **Update repository references:** 
   - Update `repo_name` in `config.toml` to match your fork (`your-username/rss-aggregator`)
   - Update the GitHub Action workflows to reference your repository
3. **Set up the PAT token** (see section below) as a repository secret named `RELEASE_TOKEN`.
4. **Add your feeds:** Add RSS/Atom feed URLs (one per line) to the `feeds.txt` file and commit the changes.
5. **Wait for automation:** The workflow will automatically:
   - Create a release with the archiver binary (triggered by your push)
   - Run the archiver hourly to update feeds and generate the OPML file
6. **Import OPML:** Import the generated `feeds/master.opml` file into your RSS reader to access all your archived feeds with complete history.

## Configuration

The `config.toml` file allows you to configure the RSS archiver:

- `max_items`: Maximum number of items to archive per feed. Set to `0` for unlimited items (default: 300 if not specified). This controls how many articles are preserved in each archived feed.
- `repo_name`: GitHub repository name in format `owner/repo` (optional, default: "xavwe/rss-aggregator"). Used for generating URLs to your archived feeds in the OPML file.

## Setting Up the Project with a PAT

For the GitHub Actions workflows to successfully create releases and push updates (such as updating archived feeds and `feeds/master.opml`), you need to configure a Personal Access Token (PAT) and add it as a secret named `RELEASE_TOKEN` in your repository. This token is used by the workflows to authenticate operations that modify the repository.

### Required PAT Permissions

Your PAT must have the following scopes:
- **repo** — Grants full control of private repositories (and is sufficient for public repositories as well). This scope allows the workflows to:
  - Create and upload release assets.
  - Push changes (for example, updating archived feeds and `feeds/master.opml`).

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

## How It Works

### Automated Workflow
1. **Release Creation:** When you push changes, GitHub Actions automatically builds the Rust binary and creates a new release
2. **Hourly Updates:** Every hour (and after each new release), the "Update Master Feed" workflow:
   - Downloads the latest release binary
   - Runs it to fetch all RSS feeds and update archives
   - Commits any changes back to the repository

### Feed Processing
1. **Feed Fetching:** The archiver reads URLs from `feeds.txt` and fetches each RSS/Atom feed concurrently
2. **Content Preservation:** Unlike original feeds that typically only show recent items, all fetched articles are preserved in individual XML files
3. **OPML Generation:** Creates a master OPML file where `xmlUrl` points to your archived feeds (what RSS readers fetch) and `htmlUrl` points to original sources (for reference)
4. **RSS Reader Integration:** Import the OPML into any RSS reader to subscribe to your complete archived feeds instead of the limited original ones

Enjoy archiving your RSS feeds and never lose an article again! 🚀

## Credits

This project is forked from [therecluse26/rss-aggregator](https://github.com/therecluse26/rss-aggregator) and has been modified to focus on RSS feed archiving rather than aggregation.
