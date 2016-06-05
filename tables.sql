CREATE TABLE `user` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `username` varchar(32) NOT NULL,
  `email` varchar(64) NOT NULL,
  `password` varchar(32) NOT NULL,
  `salt` varchar(32) NOT NULL,
  `create_time` datetime NOT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `email` (`email`),
  UNIQUE KEY `username` (`username`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

CREATE TABLE `article` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `category` tinyint(4) NOT NULL DEFAULT '0',
  `title` varchar(64) NOT NULL,
  `content` text NOT NULL,
  `user_id` int(11) NOT NULL,
  `comments_count` int(11) NOT NULL DEFAULT '0',
  `create_time` datetime NOT NULL,
  `status` tinyint(4) NOT NULL DEFAULT '0',
  `priority` tinyint(4) NOT NULL DEFAULT '0',
  PRIMARY KEY (`id`),
  KEY `user_id` (`user_id`),
  FOREIGN KEY (`user_id`) REFERENCES `user` (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

CREATE TABLE `comment` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `article_id` int(11) NOT NULL,
  `user_id` int(11) NOT NULL,
  `content` text NOT NULL,
  `create_time` datetime NOT NULL,
  PRIMARY KEY (`id`),
  KEY `article_id` (`article_id`),
  KEY `user_id` (`user_id`),
  FOREIGN KEY (`article_id`) REFERENCES `article` (`id`),
  FOREIGN KEY (`user_id`) REFERENCES `user` (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;

CREATE TABLE `message` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `article_id` int(11) DEFAULT NULL,
  `comment_id` int(11) DEFAULT NULL,
  `from_user_id` int(11) DEFAULT NULL,
  `to_user_id` int(11) NOT NULL,
  `content` text,
  `mode` tinyint(4) NOT NULL DEFAULT '0',
  `status` tinyint(4) NOT NULL DEFAULT '0',
  `create_time` datetime NOT NULL,
  PRIMARY KEY (`id`),
  KEY `article_id` (`article_id`),
  KEY `from` (`from_user_id`),
  KEY `to` (`to_user_id`),
  KEY `comment_id` (`comment_id`),
  FOREIGN KEY (`article_id`) REFERENCES `article` (`id`),
  FOREIGN KEY (`from_user_id`) REFERENCES `user` (`id`),
  FOREIGN KEY (`to_user_id`) REFERENCES `user` (`id`),
  FOREIGN KEY (`comment_id`) REFERENCES `comment` (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8;
