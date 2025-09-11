default:
    @just --choose

# lint the client codes
lint-c:
    cd /ymym/client && \
        pnpm oxlint --fix \
            -D correctness \
            -D suspicious \
            -D pedantic \
            -D perf \
            -A group-export \
            -A no-null \
            -A filename-case \
            -A consistent-type-specifier-style \
            -A max-lines-per-function \
            -A sort-imports \
            -A prefer-global-this \
            -A func-style \
            -A sort-keys \
            -A prefer-add-event-listener \
            -A require-post-message-target-origin \
            -A new-cap \
            -A no-magic-numbers \
            -A id-length && \
        pnpm prettier -l -w \"**/*.{js,ts,vue,json,css}\" && \
        cd -

# lint the rust codes
lint-s:
    cargo check && cargo fmt && cargo clippy

# start the client dev server
dev-c:
    cd /ymym/client && pnpm nuxt dev

# start the server dev server
dev-s:
    cargo run -p yomuyume

# build the client
build-c:
    #!/usr/bin/env zsh
    cd /ymym/client && pnpm nuxt generate
    if [ -f .nuxt/dist/client/manifest.webmanifest ]; then
        cp .nuxt/dist/client/manifest.webmanifest /ymym/client/.output/public/manifest.webmanifest
    fi

# build the server
build-s +args="":
    cargo build -p yomuyume --release {{ args }}

# upgrade client dependencies
upgrade-c:
    cd /ymym/client && pnpm upgrade && cd -

# upgrade rust dependencies
upgrade-s +args="":
    cargo update {{ args }}

build:
    just build-c && just build-s

_gen-seed-file:
    #!/usr/bin/env python3
    import os
    import random
    import json
    from datetime import datetime

    # —————— NanoID implementation ——————
    LENGTH = 21
    CHARSET = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_"

    def nanoid():
        """Generate a random nanoid."""
        rnd = os.urandom(LENGTH)
        return "".join(CHARSET[b & 63] for b in rnd)

    # —————— Configurable ranges ——————
    CAT_MIN, CAT_MAX = 5, 10
    TIT_PER_CAT_MIN, TIT_PER_CAT_MAX = 3, 5
    ORPH_TIT_MIN, ORPH_TIT_MAX = 5, 10
    CHAP_MIN, CHAP_MAX = 1, 5
    PAGES_MIN, PAGES_MAX = 5, 10

    # —————— Generate categories ——————
    categories = []
    for i in range(1, random.randint(CAT_MIN, CAT_MAX) + 1):
        cid = nanoid()
        categories.append({
            'id': cid,
            'path': f"/path/to/{cid}",
            'name': f"Category {i}",
            'description': f"Description for Category {i}",
        })

    # —————— Generate titles ——————
    titles = []
    for cat in categories:
        for _ in range(random.randint(TIT_PER_CAT_MIN, TIT_PER_CAT_MAX)):
            tid = nanoid()
            titles.append({
                'id': tid,
                'category': cat['id'],
                'path': f"{cat['path']}/{tid}",
                'last_modified': 'CURRENT_TIMESTAMP',
                'comic_info': json.dumps({'info': tid}),
            })
    # orphans
    for _ in range(random.randint(ORPH_TIT_MIN, ORPH_TIT_MAX)):
        tid = nanoid()
        titles.append({
            'id': tid,
            'category': None,
            'path': f"/path/to/{tid}",
            'last_modified': 'CURRENT_TIMESTAMP',
            'comic_info': json.dumps({'info': tid}),
        })

    # —————— Generate chapters ——————
    chapters = []
    for t in titles:
        for num in range(1, random.randint(CHAP_MIN, CHAP_MAX) + 1):
            cid = nanoid()
            chapters.append({
                'id': cid,
                'title_id': t['id'],
                'path': f"{t['path']}/{cid}",
                'number': float(num),
                'last_modified': 'CURRENT_TIMESTAMP',
            })

    # —————— Generate pages ——————
    pages = []
    for ch in chapters:
        n_pages = random.randint(PAGES_MIN, PAGES_MAX)
        for page_num in range(1, n_pages + 1):
            pid = nanoid()
            width = random.randint(600, 1200)
            height = random.randint(800, 1600)
            color = "#%06x" % random.randint(0, 0xFFFFFF)
            size = random.randint(10000, 200000)
            pages.append({
                'id': pid,
                'chapter_id': ch['id'],
                'path': f"{ch['path']}/{pid}.jpg",
                'width': width,
                'height': height,
                'avg_hex_color': color,
                'size': size,
                'page_number': page_num,
            })

    # —————— Build mappings for covers ——————
    chap_to_pages = {}
    for p in pages:
        chap_to_pages.setdefault(p['chapter_id'], []).append(p['id'])
    # Add page count to each chapter
    for ch in chapters:
        ch['page_count'] = len(chap_to_pages.get(ch['id'], []))

    title_to_pages = {}
    for ch in chapters:
        title_to_pages.setdefault(ch['title_id'], [])
    for cid, pids in chap_to_pages.items():
        t_id = next(ch['title_id'] for ch in chapters if ch['id'] == cid)
        title_to_pages[t_id].extend(pids)

    # —————— Generate covers ——————
    title_covers = [
        {'title_id': t_id, 'page_id': random.choice(pids)}
        for t_id, pids in title_to_pages.items() if pids
    ]
    chapter_covers = [
        {'chapter_id': cid, 'page_id': random.choice(pids)}
        for cid, pids in chap_to_pages.items() if pids
    ]

    # —————— Generate users, sessions, collections, reading_progress ——————
    users = []
    sessions = []
    collections = []
    collection_titles = []
    reading_progress = []

    all_title_ids = [t['id'] for t in titles]
    for u_idx in range(1, 4):
        uid = nanoid()
        users.append({
            'id': uid,
            'username': f"user{u_idx}",
            'email': f"user{u_idx}@example.com",
            'password_hash': nanoid(),
            'created_at': 'CURRENT_TIMESTAMP',
            'verified_at': None,
        })
        # 2 sessions per user
        for _ in range(2):
            sessions.append({
                'secret': nanoid(),
                'user_id': uid,
                'created_at': 'CURRENT_TIMESTAMP',
            })
        # 5 collections per user
        for c_i in range(1, 6):
            col_id = nanoid()
            collections.append({
                'id': col_id,
                'name': f"Collection {c_i}",
                'user_id': uid,
                'created_at': 'CURRENT_TIMESTAMP',
            })
            # 5 random titles in each
            for tid in random.sample(all_title_ids, 5):
                collection_titles.append({
                    'collection_id': col_id,
                    'title_id': tid,
                })
        # reading progress (up to 10 titles)
        for tid in random.sample(all_title_ids, min(len(all_title_ids), 10)):
            ch_ids = [c['id'] for c in chapters if c['title_id'] == tid]
            cid_sel = random.choice(ch_ids) if ch_ids else None
            # select a page and get its page number
            if cid_sel and chap_to_pages.get(cid_sel):
                pid_sel = random.choice(chap_to_pages[cid_sel])
                page_num = next((p['page_number'] for p in pages if p['id'] == pid_sel), None)
            else:
                page_num = None
            reading_progress.append({
                'user_id': uid,
                'title_id': tid,
                'chapter_id': cid_sel,
                'page_number': page_num,
                'updated_at': 'CURRENT_TIMESTAMP',
            })

    # —————— Emit seed.sql ——————
    sections = []

    def make_insert(table, cols, rows):
        fmt_rows = []
        for r in rows:
            vals = []
            for c in cols:
                v = r.get(c)
                if v is None:
                    vals.append('NULL')
                elif isinstance(v, str) and v.startswith('CURRENT_TIMESTAMP'):
                    vals.append(v)
                # raw blob literal: unquoted X'...' should be inserted as-is
                elif isinstance(v, str) and v.startswith("X'") and v.endswith("'"):
                    vals.append(v)
                else:
                    vals.append(f"'{v}'")
            fmt_rows.append(f"    ({', '.join(vals)})")
        return f"INSERT INTO {table} ({', '.join(cols)}) VALUES\n" + ",\n".join(fmt_rows) + ";\n"

    sections.append(make_insert('categories', ['id','path','name','description'], categories))
    sections.append(make_insert(
        'titles',
        ['id','category_id','path','last_modified','comic_info'],
        [
            {**t, 'category_id': t.get('category'),
            'comic_info': f"X'{t['comic_info'].encode('utf-8').hex()}'"
            }
            for t in titles
        ]
    ))
    sections.append(make_insert('chapters', ['id','title_id','path','number','last_modified','page_count'], chapters))
    sections.append(make_insert('pages', ['id','chapter_id','path','width','height','avg_hex_color','size','page_number'], pages))
    sections.append(make_insert('title_covers', ['title_id','page_id'], title_covers))
    sections.append(make_insert('chapter_covers', ['chapter_id','page_id'], chapter_covers))
    sections.append(make_insert('users', ['id','username','email','password_hash','created_at','verified_at'], users))
    sections.append(make_insert('sessions', ['secret','user_id','created_at'], sessions))
    sections.append(make_insert('collections', ['id','name','user_id','created_at'], collections))
    sections.append(make_insert('collection_titles', ['collection_id','title_id'], collection_titles))
    sections.append(make_insert('reading_progress', ['user_id','title_id','chapter_id','page_number','updated_at'], reading_progress))

    with open('/tmp/ymym-seed.sql', 'w') as f:
        f.write(f"-- Auto-generated seed file: {datetime.now().isoformat()}\n\n")
        f.write(f"BEGIN TRANSACTION;\n")
        f.write("\n".join(sections))
        f.write(f"\nCOMMIT;\n")

    print(f"Generated seed.sql with:")
    print(f"  {len(categories)} categories")
    print(f"  {len(titles)} titles")
    print(f"  {len(chapters)} chapters")
    print(f"  {len(pages)} pages")
    print(f"  {len(title_covers)} title covers, {len(chapter_covers)} chapter covers")
    print(f"  {len(users)} users, {len(sessions)} sessions")
    print(f"  {len(collections)} collections, {len(collection_titles)} collection_titles")
    print(f"  {len(reading_progress)} reading_progress entries")


# Seed the database with random data
seed-db: _gen-seed-file
    #!/bin/zsh
    time sqlite3 /ymym/library/yomuyume.db ".read /tmp/ymym-seed.sql"
    rm -f /tmp/ymym-seed.sql