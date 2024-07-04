-- This file should undo anything in `up.sql`
-- DropForeignKey
ALTER TABLE "CloudUser" DROP CONSTRAINT "CloudUser_userId_fkey";

-- DropForeignKey
ALTER TABLE "UserRole" DROP CONSTRAINT "UserRole_roleId_fkey";

-- DropForeignKey
ALTER TABLE "UserRole" DROP CONSTRAINT "UserRole_userId_fkey";

-- DropIndex
DROP INDEX "Role_name_key";

-- DropIndex
DROP INDEX "User_email_key";

-- DropIndex
DROP INDEX "User_username_key";

-- DropTable
DROP TABLE "CloudUser";

-- DropTable
DROP TABLE "UserRole";

-- DropTable
DROP TABLE "Role";

-- DropTable
DROP TABLE "User";

-- DropEnum
DROP TYPE "CloudProvider";

-- DropEnum
DROP TYPE "LoginProvider";