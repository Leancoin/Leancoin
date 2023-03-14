export const isBetween1and5 = (): boolean => {
    const currentDate = new Date();
    const currentDay = currentDate.getDate();
    return currentDay >= 1 && currentDay <= 5;
};
