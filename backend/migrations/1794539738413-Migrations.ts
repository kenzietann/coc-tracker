import { MigrationInterface, QueryRunner } from "typeorm";

export class Migrations1794539738413 implements MigrationInterface {
    name = 'Migrations1794539738413'

    public async up(queryRunner: QueryRunner): Promise<void> {
        await queryRunner.query(`CREATE TABLE \`villages\` (\`id\` uuid NOT NULL, \`tag\` varchar(32) NOT NULL, \`label\` varchar(100) NULL, \`exportTimestamp\` bigint NOT NULL, \`rawExport\` text NOT NULL, UNIQUE INDEX \`IDX_0e586bfac685ac8c99073e9b07\` (\`tag\`), PRIMARY KEY (\`id\`)) ENGINE=InnoDB`);
    }

    public async down(queryRunner: QueryRunner): Promise<void> {
        await queryRunner.query(`DROP INDEX \`IDX_0e586bfac685ac8c99073e9b07\` ON \`villages\``);
        await queryRunner.query(`DROP TABLE \`villages\``);
    }

}
