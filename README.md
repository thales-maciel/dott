# dotr - The Dotfile Manager You Never Knew You Didn't Need

Dotr is here to manage your dotfiles in the simplest way it knows how.

## Why dotr? Seriously, Why?

Dotr aims to be straightforward and unobtrusive. It's *not* going to revolutionize your workflow. The goal is to try a different approach to dotfile management by keeping your dotfiles where they belong, without __symlinks__.

## Getting Started, Or Something Like That

Joining the Dotr party is as easy as falling off a log:

1. Kick things off with `dotr init`. This will create a fresh git repo.
2. Give dotr a list of globs in your `$HOME/.config/dotr/dotr.config` file. Globs are all relative to your home folder.
3. Run `dotr sync` to *copy* all matching files to your repo, keeping your home folder's original structure intact.
4. Run `dotr cd` to go to the git repo. Manage your dotfiles using git. Do whatever you want. Dotr is not your mom.
5. Use `dotr install` to copy the files from the repo back to their home.

Dotr is a simple, beta, lightweight solution. And by that i mean it's a work in progress. Got a cool idea for dotr? Feel free to make a pull request.

## FAQ
- *How to ignore files?*
    Use a .gitignore file. All globs are relative to your home folder. I wonder if this was a good idea...
- *Wheres the repo?*
    Run `dotr cd` followed by `pwd`.
- *Can i change the repo location?*
    No. Unless you can change my mind about it.
- *Why rust?*
    Because it's a side project, it should be funny. Besides that, I really wanted to test `nix flakes`


Thanks for giving Dotr a look.

