#include "../THttpSink.h"

#include <gtest/gtest.h>
#include <QDebug>

TEST(ItemToJson, ErrorOnly) {
   const QString Actual = ItemToJson({.ErrorString = "asdfqwer"});
   ASSERT_TRUE(Actual.contains("\"ErrorString\""));
   ASSERT_TRUE(Actual.contains("\"asdfqwer\""));
   ASSERT_TRUE(Actual.contains("\"Time\""));
}

TEST(ItemToJson, OneNameTemp) {
   const QString Actual = ItemToJson({.NameToTemp = {{"asdfqwer", 12345.6789}}});
   ASSERT_TRUE(Actual.contains("\"asdfqwer\""));
   ASSERT_TRUE(Actual.contains("12345.6789"));
}

TEST(ItemToJson, TwoNameTemp) {
   const QString Actual = ItemToJson({.NameToTemp = {{"asdfqwer", 12345.6789}, {"zxcv", -4}}});
   ASSERT_TRUE(Actual.contains("\"asdfqwer\""));
   ASSERT_TRUE(Actual.contains("12345.6789"));
   ASSERT_TRUE(Actual.contains("\"zxcv\""));
   ASSERT_TRUE(Actual.contains("-4"));
   ASSERT_TRUE(Actual.contains("\"Time\""));
}
