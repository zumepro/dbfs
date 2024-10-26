SET SQL_MODE = "NO_AUTO_VALUE_ON_ZERO";
START TRANSACTION;
SET time_zone = "+00:00";

CREATE TABLE `block` (
  `inode_id` int(10) UNSIGNED NOT NULL,
  `block_id` int(10) UNSIGNED NOT NULL,
  `data` blob NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `file` (
  `parent_inode_id` int(10) UNSIGNED NOT NULL,
  `name` varchar(255) NOT NULL,
  `inode_id` int(10) UNSIGNED NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `file_types` (
  `id` char(1) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL,
  `description` varchar(50) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;

CREATE TABLE `group` (
  `id` int(10) UNSIGNED NOT NULL,
  `name` varchar(255) DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `inode` (
  `id` int(10) UNSIGNED NOT NULL,
  `owner` int(10) UNSIGNED NOT NULL,
  `group` int(10) UNSIGNED NOT NULL,
  `file_type` char(1) NOT NULL,
  `special_bits` tinyint(4) UNSIGNED NOT NULL DEFAULT 0,
  `user_perm` tinyint(4) UNSIGNED NOT NULL DEFAULT 0,
  `group_perm` tinyint(4) UNSIGNED NOT NULL DEFAULT 0,
  `other_perm` tinyint(4) UNSIGNED NOT NULL DEFAULT 0,
  `created_at` timestamp NOT NULL DEFAULT current_timestamp(),
  `modified_at` timestamp NOT NULL DEFAULT current_timestamp() ON UPDATE current_timestamp(),
  `accessed_at` timestamp NOT NULL DEFAULT current_timestamp()
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

CREATE TABLE `permissions` (
  `id` tinyint(4) UNSIGNED NOT NULL,
  `can_read` tinyint(1) UNSIGNED NOT NULL,
  `can_write` tinyint(1) UNSIGNED NOT NULL,
  `can_execute` tinyint(1) UNSIGNED NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;

CREATE TABLE `special_bits` (
  `id` tinyint(4) UNSIGNED NOT NULL,
  `setuid` tinyint(1) UNSIGNED NOT NULL,
  `setgid` tinyint(1) UNSIGNED NOT NULL,
  `sticky` tinyint(1) UNSIGNED NOT NULL,
  `description` varchar(100) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;

CREATE TABLE `user` (
  `id` int(10) UNSIGNED NOT NULL,
  `name` varchar(255) DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;


ALTER TABLE `block`
  ADD PRIMARY KEY (`inode_id`,`block_id`);

ALTER TABLE `file`
  ADD PRIMARY KEY (`parent_inode_id`,`name`),
  ADD KEY `inode_id` (`inode_id`);

ALTER TABLE `file_types`
  ADD PRIMARY KEY (`id`);

ALTER TABLE `group`
  ADD PRIMARY KEY (`id`);

ALTER TABLE `inode`
  ADD PRIMARY KEY (`id`);

ALTER TABLE `permissions`
  ADD PRIMARY KEY (`id`);

ALTER TABLE `special_bits`
  ADD PRIMARY KEY (`id`);

ALTER TABLE `user`
  ADD PRIMARY KEY (`id`);


ALTER TABLE `group`
  MODIFY `id` int(10) UNSIGNED NOT NULL AUTO_INCREMENT;

ALTER TABLE `inode`
  MODIFY `id` int(10) UNSIGNED NOT NULL AUTO_INCREMENT;

ALTER TABLE `user`
  MODIFY `id` int(10) UNSIGNED NOT NULL AUTO_INCREMENT;


ALTER TABLE `block`
  ADD CONSTRAINT `block_inode` FOREIGN KEY (`inode_id`) REFERENCES `inode` (`id`);

ALTER TABLE `file`
  ADD CONSTRAINT `file_inode` FOREIGN KEY (`inode_id`) REFERENCES `inode` (`id`),
  ADD CONSTRAINT `file_parent_inode` FOREIGN KEY (`parent_inode_id`) REFERENCES `inode` (`id`);

ALTER TABLE `inode`
  ADD CONSTRAINT `inode_file_type` FOREIGN KEY (`file_type`) REFERENCES `file_types` (`id`),
  ADD CONSTRAINT `inode_group` FOREIGN KEY (`group`) REFERENCES `group` (`id`),
  ADD CONSTRAINT `inode_group_perm` FOREIGN KEY (`group_perm`) REFERENCES `permissions` (`id`),
  ADD CONSTRAINT `inode_other_perm` FOREIGN KEY (`other_perm`) REFERENCES `permissions` (`id`),
  ADD CONSTRAINT `inode_owner` FOREIGN KEY (`owner`) REFERENCES `user` (`id`),
  ADD CONSTRAINT `inode_special_bits` FOREIGN KEY (`special_bits`) REFERENCES `special_bits` (`id`),
  ADD CONSTRAINT `inode_user_perm` FOREIGN KEY (`user_perm`) REFERENCES `permissions` (`id`);
COMMIT;
