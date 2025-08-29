# matt.boats

Generate pages using
```bash
$ rm -r dist/ && python3.12 make_page.py 0*
```

Then host
```bash
$ python3 -m http.server 8000 -d dist/
```

Pushes to main will deploy automatically!
