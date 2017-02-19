#!/usr/bin/env python3

from gevent import monkey
monkey.patch_all()

import logging
import os
import datetime

from gevent.pool import Pool
import requests
import redis
from jinja2 import Environment, FileSystemLoader, select_autoescape

API_PREFIX = "https://hacker-news.firebaseio.com/v0"
REDIS_KEY = "rust-lang-cn:hacker-news"
MAX_COUNT = 20
STATIC_PATH = "http://static.liyuan.im/rust-lang-cn"

redis = redis.StrictRedis()

def fetch_story(story_id):
    return requests.get(
        "{}/item/{}.json".format(API_PREFIX, story_id)).json()


def main():
    top_story_ids = requests.get(
        "{}/topstories.json".format(API_PREFIX)).json()
    pool = Pool(50)
    rust_stories = list(
        filter(lambda story: "Rust" in story.get("title", ""),
               pool.imap(fetch_story, top_story_ids)))[:MAX_COUNT]
    stories_length = len(rust_stories)
    if stories_length < MAX_COUNT:
        existed_story_ids = set(map(int, redis.lrange(REDIS_KEY, 0, -1)))
        existed_story_ids -= set(item["id"] for item in rust_stories)
        rust_stories.extend(
            pool.imap(fetch_story,
                      list(existed_story_ids)[:MAX_COUNT - stories_length]))
    redis.lpush(REDIS_KEY, *[item["id"] for item in rust_stories])
    redis.ltrim(REDIS_KEY, 0, MAX_COUNT - 1)
    render(rust_stories)


def render(rust_stories):
    file = os.path.realpath(__file__)
    dir = os.path.split(file)[0]
    dir = os.path.split(dir)[0]

    env = Environment(
        loader=FileSystemLoader('{}/tools/templates'.format(dir)),
        autoescape=select_autoescape(['html'])
    )

    for story in rust_stories:
        story["datetime"] = datetime.datetime.fromtimestamp(
            story["time"]).strftime("%Y-%m-%d %H:%M:%S")

    template = env.get_template('hacker_news.html')
    content = template.render(
        static_path=STATIC_PATH,
        stories=rust_stories,
        )
    print(content)


if __name__ == "__main__":
    main()
