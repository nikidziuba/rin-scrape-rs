# Rin Scraper

This is a tool that scrapes cs.rin.ru SCS (Steam Content Sharing) forum for photos and links from game posts. The final idea is to create a auto-updater for games that don't have DRM other than the Steam one 


# Current Progress

Currently, it can only get image and display it in a terminal, steam and download (GDrive etc.) links.

## Warning

The terminal that you want to use has to have true color and hyperlinks, **default Windows Command Line will not work**.
I personally use [Windows Terminal](https://www.microsoft.com/store/productId/9N0DX20HK701) on Windows 10, but any modern terminal shold work.

## Building from Source
You should first install Rust from [here](https://www.rust-lang.org/tools/install) and make sure you have git installed.
Then:

    git clone https://github.com/nikidziuba/rin-scrape-rs.git
    cd rin-scrape-rs
Create a .env file with your rin username and password e.g:
	

    USERNAME=example
    PASSWORD=12345678

After that you can run it by:

    cargo run  
or
   

     cargo run --release
See Rust documentation for more info

# Prerequisites

 - A Modern Terminal supporting true color and hyperlinks
 - geckodriver in PATH (It is currently used for authorization, will probably be changed to HTTP requests

# How does it look?
Here's an example - output of "cargo run 1229490":
![](/images/example.png)
 

