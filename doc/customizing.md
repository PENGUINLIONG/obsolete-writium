# Customizing

In this article, I will elaborate how you can utilize Writus for more customized reading experience.

Overall, Writus requires everything to be set up in compile-time. So, before you `cargo run` the project, you should finish the options listed in `settings.rs`.

Before you discussing the details, let's start with the directory structure of Writus.

## Directory Structure

In `settings.rs`, you will need to provide a existing directory to have your essays, static files (like css, js, etc), and templates stored. Let's call it `./`. Then, we will have:

```
./ --+-- error
     +-- post
     +-- static
     +-- template
```

Every request is checked for extension to ensure the requested file is allowed to be distributed.

## Error Handling

When a requested file does not belong to a certain catagory, or does not exist at all, an error will be responsed. Writus allows you to make your own error reporting page. For HTTP error `404 Not Found`, place file `404.html` here. Any unknown error will be redirected to `Unknown.html`.

## Posts

Posts are written in common Markdown and encoded in UTF-8. Place them in `./post`. Each post, named `content.md`, is place in an individual sub-directory with its metadata. File search is case sensitive.

## Static Files

Static resources like `*.css` and `*.js` are placed in `./static`.

## Templates

Templates are used to decorate distributed contents. We use special HTML processing instructions, enclosed by `<?` and `?>`, to denote variables and external html fragments.

You can inline external html fragments with:

```
<?frag [relative path] ?>
```

To insert runtime variables, write:

```
<?var [variable name] ?>
```

Except that variable `content` is direcly derived from the content of `content.md`, other variables have to be manually defined by users, in file `metadata.json`. File search and variable name are case sensitive. The following variables are recommended to be defined and will be set with default value in case of absence. Parse error born by `metadata.json` will be ignored, and will not prevent Writus from working properly.

`author`: Name or alias of post author. [default: `Akari`]
`content`: Content of the article, the content of `content.md`. (Already defined)
`pub-date`: Publish date. [default: File creation date. ]
`title`: Title of article. [default: `Untitled`]
