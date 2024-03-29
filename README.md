# Blog v3

This repository contains the source code for the third iteration of my personal blog. The blog is built in pure TypeScript, statically generated using Astro.

## Project Structure

The project is organized into several key directories and files, each with a specific purpose:

- **public**: This directory contains all the static assets that are served directly by the server. This includes images, fonts, and other media.

- **src**

    - **components**: This is where the reusable building blocks of the application are kept. Each component represents a self-contained piece of UI.

    - **content**: This directory contains all the static content, such as text, images, or other media. If it's something that users see but doesn't change, it's likely found here.

        - **blog**: This directory contains the markdown files that represent the blog posts. Each file corresponds to a different blog post.

    - **layouts**: This is where the common layouts for the pages are defined. These are the consistent parts of the UI that provide a unified look and feel across the application.

    - **pages**: This directory contains the components that make up entire pages in the application. Each file corresponds to a different page that users can visit.

    - **utils**: This directory contains utility functions that are used across the application.

## Collaboration

As with the original blog, this repository, *compared to other repositories of mine*, isn't meant to be welcoming to contributions from other developers given the nature of a *personal* blog, it is however open source with the goal of inspiring people interested in the behind the scene magic of the site and I do welcome people finding spelling mistakes, etc.