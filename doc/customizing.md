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

Posts are written in common Markdown, place them in `./post`.

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

The following variables are provided:

`Title`: First line of the article. Any preceeding hash sign will be removed.
`Content`: Content of the article, starting from the 2nd line.
