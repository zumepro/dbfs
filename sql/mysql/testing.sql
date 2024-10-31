DROP TABLE IF EXISTS `test`;
CREATE TABLE `test` (
  `id` int(11) DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;


DROP TABLE IF EXISTS `test_prepared`;
CREATE TABLE `test_prepared` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `test_name` char(3) DEFAULT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB AUTO_INCREMENT=4 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_uca1400_ai_ci;

LOCK TABLES `test_prepared` WRITE;
INSERT INTO `test_prepared` VALUES
(1,'aaa'),
(2,'bbb'),
(3,'ccc');
UNLOCK TABLES;
