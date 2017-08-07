# Customizing

In this article, I will elaborate how you can utilize Writus for more customized
writing experience.

Writus read a configuration file to extract information it needs. Assume the
file is called `settings.json`, we must have the following items set:

- `hostAddr`: Host server address or domain. Must include port number.
- `postDir`: The directory where posts located.
- `errorDir`: The directory where error pages located.
- `templateDir`: The directory where template files located.
- `staticDir`: The directory where static resources located.
- `rootDir`: The directory where the root path directly mapped to.
- `cacheDir`: The directory where cache is output.

The following items are optional:

- `digestTemplatePath`: Digest template file path in $TEMPLATE_DIR. MUST NOT
have slash as prefix. [default: digest.html]
- `indexTemplatePath`: Index template file path in $TEMPLATE_DIR. MUST NOT have
slash as prefix. [default: index.html]
- `paginationTemplatePath`: Pagination template file path in $TEMPLATE_DIR. MUST
NOT have slash as prefix. [default: pagination.html]
- `postTemplatePath`: Post template file path in $TEMPLATE_DIR. MUST NOT have
slash as prefix. [default: post.html]
- `digestsPerPage`: Number of digests shown per page on index page. [default: 5]

The file will be read once during initialization, and will not be accessed a
second time.

To start Writus up, run the following commands:

```
cd /path/to/Writus
cargo build --release
cargo run --release "/path/to/settings.json"
```

Note: You can have different websites using different combinations of resources
running on a single physical server using different configuration files. But you
have to separate the caches and initialize them one-by-one.

## Directory Structure

The number of directory exposed is limited to three: `root`, `post`, and
`static`. The other directories are consumed by Writus itself. The directories
can, but not necessarily stay in a same place.

```
/(root) --+-- post
          +-- static
```

A request towards a file in an exposed first-layer directory will be treated
as-is.

However, when a incoming URL try accessing a first-layer directory other than
`post` and `static`, like `/not/exposed.txt`, it will be treated as a request
towards something in `root`. So the actual file searching will be redirected to
`/root/not/exposed.txt`. All requests towards the root of exposed first-layer
directories will be responded with `403 Forbidden`.

Additionally, every request is checked for extension to ensure the requested
file is allowed to be distributed. See [Safety](/doc/safety.md) for more
information.

## Error Handling

When a requested file does not belong to a certain catagory, or does not exist
at all, an error will be responsed. Writus allows you to make your own error
reporting page. For HTTP error `404 Not Found`, place file `404.html` here. Any
unknown error will be redirected to `Unknown.html`.

## Posts

Posts are written in common Markdown and encoded in UTF-8. Place them in
`./post`. Each post, named `content.md`, is place in an individual sub-directory
with its metadata. File search is case-sensitive, although path search on
Windows are not case-sensitive due to historical reasons.

## Static Files

Static resources like `*.css` and `*.js` are placed in `./static`.

## Templates

Templates are used to decorate distributed contents. We use special HTML
processing instructions(PI), enclosed by `<?` and `?>`, to denote variables and
external html fragments.

You can inline external html fragments with:

```
<?frag [absolute path in template directory] ?>
```

To insert template variables, write:

```
<?var [variable name] ?>
```

The PI are simply replaced rather than being checked following (X)HTML standard
strictly, so a PI can be right inside a HTML tag. For example:

```
<a href="<?var previousPageLink?>">Previous Page</a>
```

### Template Variables

Templates are filled with variables derived from diversed ways during caching,
or in-time generation.

#### Post Template

Template variables used to fill `post` are made from three different ways:
`content.md` itself, filesystem metadata of `content.md`, and `metadata.json`.

Only `content` is generated from `content.md`.

The following template variables are from filesystem metadata:

- `created`: File creation time.
- `modified`: Last modificatoin time.

The following template variables are from `metadata.json` and are optional:

- `author`: Author of article. [default: Akari]
- `title`: Title of article. [default: Untitled]
- `published`: Publish date of article. [default: Same as `created`]

File search and variable name are case-sensitive. Parse error born by
`metadata.json` will be ignored, and will not prevent Writus from working
properly.

All time variables, including following ones, should be written in string
literal, and should follow [RFC3999](https://tools.ietf.org/html/rfc3339) to be
parsed properly by Writus.

#### Index Template

Index templates are used to generate index page. Index page will be responded
with if the requested URL points to `/`.

Index templates are provided with two variables:

- `digests`: Digests of articles. Derived from digest template.
- `pagination`: Pagination. Derived from pagination template.

#### Digest Template

Digest templates are used to generate blocks of digests. The digests will be
concatenated together to form `digests` used in index template.

- `digest`: Digest of article.
- `path`: Path to the full article.

#### Pagination Template

Page indicator and page turner.

- `previousPage`: Page number of the previous page. It will be empty if the
previous page doesn't exist.
- `previousPageLink`: Link to the previous page. It will be empty if the
previous page doesn't exist.
- `thisPage`: Page number of this page. It will be empty if this page doesn't
exist.
- `nextPage`: Page number of the next page. It will be empty if the next page
doesn't exist.
- `nextPageLink`: Link to the next page. It will be empty if the next page
doesn't exist.
