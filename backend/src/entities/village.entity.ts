import { Column, Entity, PrimaryGeneratedColumn } from 'typeorm';

@Entity('villages')
export class VillageEntity {
  @PrimaryGeneratedColumn('uuid')
  id!: string;

  @Column({ type: 'varchar', length: 32, unique: true })
  tag!: string;

  @Column({ type: 'varchar', length: 100, nullable: true })
  label!: string | null;

  @Column({
    type: 'bigint',
    transformer: {
      to: (value: number) => value,
      from: (value: string | number) => Number(value),
    },
  })
  exportTimestamp!: number;

  @Column({ type: 'simple-json' })
  rawExport!: Record<string, unknown>;
}