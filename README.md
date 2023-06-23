# dotr - The Dotfile Manager You Never Knew You Didn't Need

> Dotr is here to manage your dotfiles in the simplest way it knows how... by copying things around.

## Why dotr? Seriously, Why?

Dotr aims to be straightforward and unobtrusive. It's *not* going to revolutionize your workflow. The goal is to try a different approach to dotfile management by keeping your dotfiles where they belong, __without symlinks__.

## Getting Started, Or Something Like That

Joining the Dotr party is as easy as falling off a log:

1. Kick things off by going to where you want to keep your dotfiles. **All actions should be ran inside this directory**
```bash
mkdir ~/dotfiles && cd ~/dotfiles
``` 
2. Create a `dotr.config` file and write globs into it. **Globs are all relative to your home folder**.
```bash
echo ".config/git/**/*" >> dotr.config
```
3. Run `dotr refresh` to *copy* all matching files from your home and their respective directory structure.
4. Use `dotr install` to copy the files from the repo back to their home whenever you need it.

Dotr is a simple, beta, lightweight solution. And by that i mean it's a work in progress. Got a cool idea for dotr? Feel free to make a pull request.

## FAQ
- __How does dotr know if it should delete a file?__
    - Dotr uses the patterns in dotr.config to match files from the source and from the target directories. If a file is not found in source, but found in target, dotr deletes it in order to keep consistency with the source directory that does not contain the file.
- __I'm scared of all this copying and overwriting__
    - Every Dotr command displays a list of all operations that will be performed, as well as a prompt where you can choose to cancel the execution of the command. You can also run commands with `-r` or `--raw` to perform a dry-run.


Thanks for giving Dotr a look.

