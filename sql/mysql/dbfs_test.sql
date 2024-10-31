SET SQL_MODE = "NO_AUTO_VALUE_ON_ZERO";
START TRANSACTION;
SET time_zone = "+00:00";

CREATE TABLE `block` (
  `inode_id` int(10) UNSIGNED NOT NULL,
  `block_id` int(10) UNSIGNED NOT NULL,
  `data` blob NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

INSERT INTO `block` (`inode_id`, `block_id`, `data`) VALUES
(2, 1, 0x48656c6c6f2c20776f726c64210a),
(3, 1, REPEAT(CHAR(0), 4096)),
(3, 2, REPEAT(CHAR(0), 4096)),
(3, 3, REPEAT(CHAR(0), 4096)),
(3, 4, 0x616161610a),
(5, 1, 0x77686174207765726520796f7520657870656374696e670a),
(6, 1, 0x68747470733a2f2f7777772e796f75747562652e636f6d2f77617463683f763d64517734773957675863510a),
(8, 1, 0x746573742e747874);

CREATE TABLE `file` (
  `parent_inode_id` int(10) UNSIGNED NOT NULL,
  `name` varchar(255) NOT NULL,
  `inode_id` int(10) UNSIGNED NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

INSERT INTO `file` (`parent_inode_id`, `name`, `inode_id`) VALUES
(1, '/', 1),
(1, 'test.txt', 2),
(1, 'hardlink_to_test.bin', 3),
(1, 'test.bin', 3),
(1, 'more_testing', 4),
(4, 'partially_private_file.txt', 5),
(4, 'very_private_file.txt', 6),
(4, 'empty_file.bin', 7),
(1, 'symlink_to_test.txt', 8);

CREATE TABLE `file_types` (
  `id` char(1) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL,
  `description` varchar(50) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;

INSERT INTO `file_types` (`id`, `description`) VALUES
('-', 'Regular file'),
('b', 'Block device'),
('c', 'Character device'),
('d', 'Directory'),
('l', 'Symbolic link'),
('p', 'Named pipe'),
('s', 'Socket');

CREATE TABLE `group` (
  `id` int(10) UNSIGNED NOT NULL,
  `name` varchar(255) DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

INSERT INTO `group` (`id`, `name`) VALUES
(1, 'root'),
(2, 'user');

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

INSERT INTO `inode` (`id`, `owner`, `group`, `file_type`, `special_bits`, `user_perm`, `group_perm`, `other_perm`, `created_at`, `modified_at`, `accessed_at`) VALUES
(1, 1, 1, 'd', 0, 7, 5, 5, '2024-10-24 17:52:52', '2024-10-24 17:53:10', '2024-10-24 17:52:52'),
(2, 2, 2, '-', 0, 6, 4, 4, '2024-10-24 17:54:00', '2024-10-24 17:54:00', '2024-10-24 17:54:00'),
(3, 2, 2, '-', 0, 6, 4, 4, '2024-10-24 17:56:34', '2024-10-24 17:57:14', '2024-10-24 17:56:34'),
(4, 2, 2, 'd', 0, 7, 5, 5, '2024-10-26 16:59:30', '2024-10-26 16:59:30', '2024-10-26 16:59:30'),
(5, 2, 2, '-', 0, 6, 4, 0, '2024-10-26 17:00:19', '2024-10-26 17:00:19', '2024-10-26 17:00:19'),
(6, 1, 1, '-', 0, 6, 0, 0, '2024-10-26 17:00:47', '2024-10-26 17:00:47', '2024-10-26 17:00:47'),
(7, 2, 2, '-', 0, 6, 4, 4, '2024-10-26 18:10:32', '2024-10-26 18:10:32', '2024-10-26 18:10:32'),
(8, 2, 2, 'l', 0, 6, 4, 4, '2024-10-27 08:27:06', '2024-10-27 08:27:06', '2024-10-27 08:27:06');

CREATE TABLE `permissions` (
  `id` tinyint(4) UNSIGNED NOT NULL,
  `can_read` tinyint(1) UNSIGNED NOT NULL,
  `can_write` tinyint(1) UNSIGNED NOT NULL,
  `can_execute` tinyint(1) UNSIGNED NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;

INSERT INTO `permissions` (`id`, `can_read`, `can_write`, `can_execute`) VALUES
(0, 0, 0, 0),
(1, 0, 0, 1),
(2, 0, 1, 0),
(3, 0, 1, 1),
(4, 1, 0, 0),
(5, 1, 0, 1),
(6, 1, 1, 0),
(7, 1, 1, 1);

CREATE TABLE `special_bits` (
  `id` tinyint(4) UNSIGNED NOT NULL,
  `setuid` tinyint(1) UNSIGNED NOT NULL,
  `setgid` tinyint(1) UNSIGNED NOT NULL,
  `sticky` tinyint(1) UNSIGNED NOT NULL,
  `description` varchar(100) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;

INSERT INTO `special_bits` (`id`, `setuid`, `setgid`, `sticky`, `description`) VALUES
(0, 0, 0, 0, 'No special bits'),
(1, 0, 0, 1, 'Sticky bit'),
(2, 0, 1, 0, 'Set group ID'),
(3, 0, 1, 1, 'Set group ID and Sticky bit'),
(4, 1, 0, 0, 'Set user ID'),
(5, 1, 0, 1, 'Set user ID and Sticky bit'),
(6, 1, 1, 0, 'Set user ID and Set group ID'),
(7, 1, 1, 1, 'Set user ID, Set group ID, and Sticky bit');

CREATE TABLE `user` (
  `id` int(10) UNSIGNED NOT NULL,
  `name` varchar(255) DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;

INSERT INTO `user` (`id`, `name`) VALUES
(1, 'root'),
(2, 'user');


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
  ADD PRIMARY KEY (`id`),
  ADD KEY `inode_file_type` (`file_type`),
  ADD KEY `inode_group` (`group`),
  ADD KEY `inode_group_perm` (`group_perm`),
  ADD KEY `inode_other_perm` (`other_perm`),
  ADD KEY `inode_owner` (`owner`),
  ADD KEY `inode_special_bits` (`special_bits`),
  ADD KEY `inode_user_perm` (`user_perm`);

ALTER TABLE `permissions`
  ADD PRIMARY KEY (`id`);

ALTER TABLE `special_bits`
  ADD PRIMARY KEY (`id`);

ALTER TABLE `user`
  ADD PRIMARY KEY (`id`);


ALTER TABLE `group`
  MODIFY `id` int(10) UNSIGNED NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=3;

ALTER TABLE `inode`
  MODIFY `id` int(10) UNSIGNED NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=9;

ALTER TABLE `user`
  MODIFY `id` int(10) UNSIGNED NOT NULL AUTO_INCREMENT, AUTO_INCREMENT=3;


ALTER TABLE `block`
  ADD CONSTRAINT `block_inode` FOREIGN KEY (`inode_id`) REFERENCES `inode` (`id`) ON DELETE CASCADE ON UPDATE CASCADE;

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
