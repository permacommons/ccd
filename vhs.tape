# Created with https://github.com/charmbracelet/vhs
# Run in a privileged Docker container based on Debian Trixie like so:
#
# docker run -it --cap-add=SYS_ADMIN --hostname core -v .:/vhs --security-opt seccomp=unconfined vhs /bin/bash
# 
# To get VHS running, had to install Homewbrew, git, build-essentials, and
# probably a few more things.

Set Theme "Aardvark Blue"
Output demo.gif

Hide
Type "source /home/permacommons/.bashrc && clear"
Enter
Sleep 250ms
Show

Type "ccd"
Sleep 500ms
Enter

Type "Videos"
Sleep 3s
Enter

# Admire the output for a bit.
Sleep 3s

