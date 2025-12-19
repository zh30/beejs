// 简化类测试 - 不使用类成员类型注解
class Student {
    constructor(id, grade) {
        this.studentId = id;
        this.grade = grade;
    }

    display() {
        console.log("Student ID:", this.studentId);
    }
}

const student = new Student(1001, "A");
student.display();
console.log("Grade:", student.grade);
