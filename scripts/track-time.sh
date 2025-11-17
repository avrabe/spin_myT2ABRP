#!/bin/bash
# Time tracking helper for Toyota MyT2ABRP project
# Usage: ./scripts/track-time.sh start "task description"
#        ./scripts/track-time.sh end "task description"

TRACKING_FILE="TIME_TRACKING.md"
ACTION=$1
TASK=$2

TIMESTAMP=$(date +"%H:%M:%S")
ISO_TIMESTAMP=$(date --iso-8601=seconds)

case "$ACTION" in
    start)
        echo "⏱️  Started: $TASK at $TIMESTAMP"
        echo "$ISO_TIMESTAMP|START|$TASK" >> .time_log
        ;;
    end)
        echo "✅ Completed: $TASK at $TIMESTAMP"
        echo "$ISO_TIMESTAMP|END|$TASK" >> .time_log
        ;;
    summary)
        echo "📊 Time Tracking Summary"
        echo "======================="
        if [ -f .time_log ]; then
            TOTAL_TASKS=$(grep "END" .time_log | wc -l)
            echo "Tasks completed: $TOTAL_TASKS"

            # Calculate total time
            START_TIME=$(head -1 .time_log | cut -d'|' -f1)
            END_TIME=$(tail -1 .time_log | cut -d'|' -f1)

            START_SEC=$(date -d "$START_TIME" +%s)
            END_SEC=$(date -d "$END_TIME" +%s)
            DIFF=$((END_SEC - START_SEC))

            HOURS=$((DIFF / 3600))
            MINUTES=$(((DIFF % 3600) / 60))
            SECONDS=$((DIFF % 60))

            echo "Total time: ${HOURS}h ${MINUTES}m ${SECONDS}s"
            echo "Target: 5h 0m 0s minimum"

            if [ $DIFF -ge 18000 ]; then
                echo "✅ Target reached!"
            else
                REMAINING=$((18000 - DIFF))
                R_HOURS=$((REMAINING / 3600))
                R_MINUTES=$(((REMAINING % 3600) / 60))
                echo "⏳ Remaining: ${R_HOURS}h ${R_MINUTES}m"
            fi
        else
            echo "No time log found. Start tracking with: ./scripts/track-time.sh start"
        fi
        ;;
    *)
        echo "Usage: $0 {start|end|summary} \"task description\""
        exit 1
        ;;
esac
