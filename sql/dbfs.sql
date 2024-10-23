SET SQL_MODE = "NO_AUTO_VALUE_ON_ZERO";
START TRANSACTION;
SET time_zone = "+00:00";

/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!40101 SET NAMES utf8mb4 */;


CREATE TABLE `block` (
  `inode_id` int(10) UNSIGNED NOT NULL,
  `block_id` int(10) UNSIGNED NOT NULL,
  `data` blob NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `file` (
  `id` int(10) UNSIGNED NOT NULL,
  `name` varchar(255) NOT NULL,
  `inode_id` int(10) UNSIGNED NOT NULL,
  `parent_id` int(10) UNSIGNED DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `group` (
  `id` int(10) UNSIGNED NOT NULL,
  `name` varchar(255) DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `inodes` (
  `id` int(10) UNSIGNED NOT NULL,
  `mode` char(10) NOT NULL,
  `owner` int(10) UNSIGNED NOT NULL,
  `group` int(10) UNSIGNED NOT NULL,
  `created_at` timestamp NOT NULL DEFAULT current_timestamp(),
  `modified_at` timestamp NOT NULL DEFAULT current_timestamp() ON UPDATE current_timestamp(),
  `accessed_at` timestamp NOT NULL DEFAULT current_timestamp()
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `user` (
  `id` int(10) UNSIGNED NOT NULL,
  `name` varchar(255) DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


ALTER TABLE `block`
  ADD PRIMARY KEY (`inode_id`,`block_id`);

ALTER TABLE `file`
  ADD PRIMARY KEY (`id`),
  ADD KEY `file_inode` (`inode_id`),
  ADD KEY `file_parent` (`parent_id`);

ALTER TABLE `group`
  ADD PRIMARY KEY (`id`);

ALTER TABLE `inodes`
  ADD PRIMARY KEY (`id`),
  ADD KEY `inode_owner` (`owner`),
  ADD KEY `inode_group` (`group`);

ALTER TABLE `user`
  ADD PRIMARY KEY (`id`);


ALTER TABLE `file`
  MODIFY `id` int(10) UNSIGNED NOT NULL AUTO_INCREMENT;

ALTER TABLE `group`
  MODIFY `id` int(10) UNSIGNED NOT NULL AUTO_INCREMENT;

ALTER TABLE `inodes`
  MODIFY `id` int(10) UNSIGNED NOT NULL AUTO_INCREMENT;

ALTER TABLE `user`
  MODIFY `id` int(10) UNSIGNED NOT NULL AUTO_INCREMENT;


ALTER TABLE `block`
  ADD CONSTRAINT `block_inode` FOREIGN KEY (`inode_id`) REFERENCES `inodes` (`id`);

ALTER TABLE `file`
  ADD CONSTRAINT `file_inode` FOREIGN KEY (`inode_id`) REFERENCES `inodes` (`id`),
  ADD CONSTRAINT `file_parent` FOREIGN KEY (`parent_id`) REFERENCES `file` (`id`);

ALTER TABLE `inodes`
  ADD CONSTRAINT `inode_group` FOREIGN KEY (`group`) REFERENCES `group` (`id`),
  ADD CONSTRAINT `inode_owner` FOREIGN KEY (`owner`) REFERENCES `user` (`id`);
COMMIT;

/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
