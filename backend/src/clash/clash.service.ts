import { BadRequestException, ConflictException, Injectable, NotFoundException } from '@nestjs/common';
import { CLASH_ITEM_NAMES } from './clash-item-names';
import { InjectRepository } from '@nestjs/typeorm';
import { VillageEntity } from 'src/entities/village.entity';
import { Repository } from 'typeorm';

export interface TimedEntry {
  name: string | null;
  entry: Record<string, unknown>;
}

export interface VillageImportResult {
  timestamp: number;
  alreadyImported: boolean;
  timedEntries: TimedEntry[];
}

@Injectable()
export class ClashService {

  constructor(
    @InjectRepository(VillageEntity) private readonly villageRepository: Repository<VillageEntity>,
  ){}

  async importVillage(villageData: unknown): Promise<VillageImportResult> {
    if (
      typeof villageData !== 'object' ||
      villageData === null ||
      Array.isArray(villageData)
    ) {
      throw new BadRequestException('Village export must be a JSON object');
    }

    if (
      !('tag' in villageData) ||
      typeof villageData.tag !== 'string' ||
      villageData.tag.trim().length === 0 ||
      villageData.tag.trim().length > 32
    ) {
      throw new BadRequestException('Village export must contain a player tag');
    }

    if (
      !('timestamp' in villageData) ||
      typeof villageData.timestamp !== 'number' ||
      !Number.isSafeInteger(villageData.timestamp) ||
      villageData.timestamp < 0
    ) {
      throw new BadRequestException('Village export must contain a valid timestamp');
    }

    const tag = villageData.tag.trim();
    const timestamp = villageData.timestamp;
    const rawExport = villageData as Record<string, unknown>;
    const results = this.extractTimedEntries(rawExport);

    const alreadyImported = await this.villageRepository.manager.transaction(
      async (manager) => {
        const repository = manager.getRepository(VillageEntity);
        const existing = await repository.findOne({
          where: { tag },
          lock: { mode: 'pessimistic_write' },
        });

        if (existing && timestamp < existing.exportTimestamp) {
          throw new ConflictException('A newer export has already been imported');
        }

        if (existing) {
          existing.exportTimestamp = timestamp;
          existing.rawExport = rawExport;
          await repository.save(existing);
        } else {
          await repository.save(
            repository.create({ tag, label: null, exportTimestamp: timestamp, rawExport }),
          );
        }

        return existing !== null;
      },
    );

    return {
      timestamp,
      alreadyImported,
      timedEntries: results,
    };
  }

  async getVillages() {
    const villages = await this.villageRepository.find();
    return villages.map((village) => ({
      id: village.id,
      tag: village.tag,
      label: village.label,
      exportTimestamp: village.exportTimestamp,
      timedEntries: this.extractTimedEntries(village.rawExport),
    }));
  }

  private extractTimedEntries(villageData: unknown): TimedEntry[] {
    const results: TimedEntry[] = [];

    const visit = (value: unknown): void => {
      if (Array.isArray(value)) {
        value.forEach(visit);
        return;
      }

      if (typeof value !== 'object' || value === null) return;

      const entry = value as Record<string, unknown>;
      if (typeof entry.timer === 'number' && Number.isFinite(entry.timer)) {
        const name =
          typeof entry.data === 'number' ? (CLASH_ITEM_NAMES[entry.data] ?? null) : null;
        const responseEntry = { ...entry };
        if ('lvl' in responseEntry) {
          responseEntry.level = responseEntry.lvl;
          delete responseEntry.lvl;
        }
        results.push({ name, entry: responseEntry });
      }

      for (const child of Object.values(entry)) {
        visit(child);
      }
    };

    visit(villageData);
    return results;
  }

  async updateVillageLabel(id: string, newLabel: unknown) {
    if (typeof newLabel !== 'string') {
      throw new BadRequestException('Village label must be a string');
    }
    const trimmedLabel = newLabel.trim();

    if (trimmedLabel.length === 0 || trimmedLabel.length > 100) {
      throw new BadRequestException('Village label must be 1 to 100 characters');
    }

    const villageData = await this.villageRepository.update({ id }, { label: trimmedLabel });

    if (villageData.affected === 0) {
      throw new NotFoundException('Village not found');
    }

    return {
      message: 'Village label successfully updated!',
      id,
      label: trimmedLabel,
    };
  }
}
